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
use crate::package::Package;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::fs::{create_dir_all, OpenOptions};
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
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(1))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Downloads and verifies a multi-file package.
    pub async fn download_package<F>(
        &self,
        package: &Package,
        output_dir: &Path,
        progress_cb: F,
    ) -> Result<(), Error>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let all_files = package.all_files();
        let total_bytes = package.total_size();
        let cumulative_downloaded = Arc::new(AtomicU64::new(0));
        let progress_fn = Arc::new(progress_cb);

        for file_entry in &all_files {
            let target_path = output_dir.join(&file_entry.name);

            if let Some(parent) = target_path.parent() {
                create_dir_all(parent).await?;
            }

            let cumulative_ref = Arc::clone(&cumulative_downloaded);
            let progress_fn_ref = Arc::clone(&progress_fn);

            self.download_file(file_entry, None, &target_path, move |curr_file, _total_file| {
                let current_total = cumulative_ref.load(Ordering::SeqCst) + curr_file;
                progress_fn_ref(current_total, total_bytes);
            })
            .await?;

            cumulative_downloaded.fetch_add(file_entry.file_size, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Downloads and verifies a single file entry defined within a catalog.
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
        let hashes = file_entry.decode_hashes()?;
        let total_chunks = hashes.len();
        let chunk_size = file_entry.chunk_size as u64;

        let mirrors: Vec<String> = if let Some(override_url) = url_override {
            vec![override_url.to_string()]
        } else {
            file_entry.mirror_urls()
        };

        if mirrors.is_empty() {
            return Err(Error::NoMirrorsAvailable);
        }

        let active_mirrors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(mirrors));
        let progress_fn = Arc::new(progress_cb);

        if let Some(parent) = output_path.parent() {
            create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(output_path)
            .await?;

        file.set_len(file_entry.file_size).await?;

        let downloaded_bytes = Arc::new(AtomicU64::new(0));

        for i in 0..total_chunks {
            let offset = i as u64 * chunk_size;
            let expected_size = if i == total_chunks - 1 {
                file_entry.last_chunk_size() as u64
            } else {
                chunk_size
            };

            let expected_hash = hashes[i];
            let mut success = false;

            let mirrors_clone: Arc<Mutex<Vec<String>>> = Arc::clone(&active_mirrors);

            let current_mirrors = {
                let guard = mirrors_clone.lock().unwrap();
                guard.clone()
            };

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
                            // Extract chunk payload whether server handled Range (206) or returned full body (200)
                            let chunk_bytes = if bytes.len() as u64 == expected_size {
                                bytes.to_vec()
                            } else if (bytes.len() as u64) >= offset + expected_size {
                                bytes[offset as usize..(offset + expected_size) as usize].to_vec()
                            } else {
                                continue;
                            };

                            let mut hasher = Sha256::new();
                            hasher.update(&chunk_bytes);
                            let calculated_hash: [u8; 32] = hasher.finalize().into();

                            if calculated_hash == expected_hash {
                                file.seek(std::io::SeekFrom::Start(offset)).await?;
                                file.write_all(&chunk_bytes).await?;

                                let current_total = downloaded_bytes
                                    .fetch_add(expected_size, Ordering::SeqCst)
                                    + expected_size;

                                progress_fn(current_total, file_entry.file_size);

                                success = true;
                                break;
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
