/*  src/downloader.rs  Chunked HTTP downloader and hash verification.
 *
 *  Copyright 2026 Emerge Cooperative
 *
 *  Permission is hereby granted, free of charge, to any person
 *  obtaining a copy of this software and associated documentation
 *  files (the "Software"), to deal in the Software without
 *  restriction, including without limitation the rights to use, copy,
 *  modify, merge, publish, distribute, sublicense, and/or sell copies
 *  of the Software, and to permit persons to whom the Software is
 *  furnished to do so, subject to the following conditions:
 *
 *  The above copyright notice and this permission notice shall be
 *  included in all copies or substantial portions of the Software.
 *
 *  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 *  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 *  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 *  NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 *  BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 *  ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 *  CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 *  SOFTWARE.
 *                                                                    */

use crate::error::Error;
use crate::file::File;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

/// Asynchronous downloader engine responsible for fetching range chunks
pub struct Downloader {
    client: Client,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    /// Creates a new `Downloader` instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::downloader::Downloader;
    /// use httpmock::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let server = MockServer::start();
    ///     let mock = server.mock(|when, then| {
    ///         when.method(GET).path("/ping");
    ///         then.status(200).body("pong");
    ///     });
    ///
    ///     let downloader = Downloader::new();
    ///     let ping_url = server.url("/ping");
    ///
    ///     let response = reqwest::get(&ping_url).await.unwrap();
    ///     assert_eq!(response.text().await.unwrap(), "pong");
    ///     mock.assert();
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Downloads and verifies a file entry defined within a catalog.
    ///
    /// The downloader fetches the file in chunk sizes.
    /// Each chunk's SHA-256 digest is verified against the hash.
    ///
    /// # Arguments
    /// - `file_entry`: Pointer to the parsed `<file>` entry.
    /// - `url_override`: Optional single URL override.
    /// - `output_path`: Destination file path on the local system.
    /// - `progress_cb`: Callback closure invoked on progress updates.
    ///
    /// # Examples
    ///
    /// Testing full chunk verification against a mock HTTP server:
    ///
    /// ```rust
    /// use httpmock::prelude::*;
    /// use laded::downloader::Downloader;
    /// use laded::file::File;
    /// use tempfile::NamedTempFile;

    /// #[tokio::main]
    /// async fn main() {
    ///     let server = MockServer::start();
    ///     let payload = b"hello world";

    ///     // SHA-256 of "hello world" encoded in Base64
    ///     let base64_hash =
    ///         "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

    ///     let _mock = server.mock(|when, then| {
    ///         when.method(GET)
    ///             .path("/test.bin")
    ///             .header("Range", "bytes=0-10");
    ///         then.status(200).body(payload);
    ///     });

    ///     let entry = File {
    ///         name: "test.bin".to_string(),
    ///         file_size: 11,
    ///         chunk_size: 11,
    ///         title: None,
    ///         family: None,
    ///         model_size: None,
    ///         quantization: None,
    ///         adaptation: None,
    ///         parameters: None,
    ///         description: None,
    ///         mirrors: None,
    ///         hash: base64_hash.to_string(),
    ///     };

    ///     let temp_out = NamedTempFile::new().unwrap();
    ///     let downloader = Downloader::new();
    ///     let mock_url = format!("{}/test.bin", server.base_url());

    ///     let result = downloader
    ///         .download_file(
    ///             &entry,
    ///             Some(&mock_url),
    ///             temp_out.path(),
    ///             |_curr, _total| {},
    ///         )
    ///         .await;

    ///     assert!(result.is_ok());
    ///     let downloaded_data = std::fs::read(temp_out.path()).unwrap();
    ///     assert_eq!(downloaded_data, payload);
    /// }
    /// ```
    ///
    /// Testing missing mirror error handling:
    ///
    /// ```rust
    /// use laded::downloader::Downloader;
    /// use laded::error::Error;
    /// use laded::file::File;
    /// use tempfile::NamedTempFile;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let hsh = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n"
    ///     let entry = File {
    ///         name: "no-mirrors.bin".to_string(),
    ///         file_size: 11,
    ///         chunk_size: 11,
    ///         title: None,
    ///         family: None,
    ///         model_size: None,
    ///         quantization: None,
    ///         adaptation: None,
    ///         parameters: None,
    ///         description: None,
    ///         mirrors: None,
    ///         hash: hsh.to_string(),
    ///     };
    ///
    ///     let temp_out = NamedTempFile::new().unwrap();
    ///     let downloader = Downloader::new();
    ///
    ///     let err = downloader
    ///         .download_file(&entry, None, temp_out.path(), |_, _| {})
    ///         .await
    ///         .unwrap_err();
    ///
    ///     assert!(matches!(err, Error::NoMirrorsAvailable));
    /// }
    /// ```
    pub async fn download_file<F>(
        &self,
        file_entry: &File,
        url_override: Option<&str>,
        output_path: &Path,
        progress_cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        // Decode chunk hashes embedded inside the file entry
        let hashes = file_entry.decode_hashes()?;
        let total_chunks = hashes.len();
        let chunk_size = file_entry.chunk_size as u64;

        // Resolve mirrors or apply runtime override
        let mirrors: Vec<String> = if let Some(override_url) = url_override {
            vec![override_url.to_string()]
        } else {
            file_entry.mirror_urls()
        };

        if mirrors.is_empty() {
            return Err(Error::NoMirrorsAvailable);
        }

        // Shared thread-safe handle for active mirrors
        let active_mirrors: Arc<Mutex<Vec<String>>> =
            Arc::new(Mutex::new(mirrors));
        let progress_fn = Arc::new(progress_cb);

        // Pre-allocate the target output file on disk
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(output_path)
            .await?;

        file.set_len(file_entry.file_size).await?;

        let downloaded_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));

        // Iterate through all required chunks sequentially
        for i in 0..total_chunks {
            let offset = i as u64 * chunk_size;
            let expected_size = if i == total_chunks - 1 {
                file_entry.last_chunk_size() as u64
            } else {
                chunk_size
            };

            let expected_hash = hashes[i];
            let mut success = false;

            // Cloned handle with explicit context
            let mirrors_clone: Arc<Mutex<Vec<String>>> =
                Arc::clone(&active_mirrors);

            let current_mirrors = {
                let guard = mirrors_clone.lock().unwrap();
                guard.clone()
            };

            // Attempt download from available mirror endpoints
            for mirror_url in &current_mirrors {
                let range_header = format!(
                    "bytes={}-{}",
                    offset,
                    offset + expected_size - 1
                );
                let response = self
                    .client
                    .get(mirror_url)
                    .header("Range", range_header)
                    .send()
                    .await;

                if let Ok(res) = response {
                    if res.status().is_success() {
                        if let Ok(bytes) = res.bytes().await {
                            if bytes.len() as u64 == expected_size {
                                // Compute SHA-256 digest of payload
                                let mut hasher = Sha256::new();
                                hasher.update(&bytes);
                                let calculated_hash: [u8; 32] =
                                    hasher.finalize().into();

                                // Verify match against hash manifest
                                if calculated_hash == expected_hash {
                                    file.seek(std::io::SeekFrom::Start(offset))
                                        .await?;
                                    file.write_all(&bytes).await?;

                                    let current_total = downloaded_bytes
                                        .fetch_add(
                                            expected_size,
                                            std::sync::atomic::Ordering::SeqCst,
                                        )
                                        + expected_size;
                                    progress_fn(
                                        current_total,
                                        file_entry.file_size,
                                    );

                                    success = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if !success {
                return Err(Error::ChunkDownloadFailed(i));
            }
        }

        file.flush().await?;
        Ok(())
    }
}


