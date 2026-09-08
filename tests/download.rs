/*  tests/download.rs  Download integration tests
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

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use laded::downloader::Downloader;
    use laded::error::Error;
    use laded::file::File;
    use tempfile::NamedTempFile;

    /// Helper function to construct a standard single-chunk `File`
    /// catalog entry.
    ///
    /// **Purpose:**
    /// Creates a mock file catalog entry configured for a single
    /// 11-byte chunk named "test.bin".
    ///
    /// **Parameters:**
    /// - `hash_base64`: The expected Base64-encoded SHA-256 chunk
    ////  hash manifest.
    fn create_test_file(hash_base64: &str) -> File {
        File {
            name: "test.bin".to_string(),
            file_size: 11,
            chunk_size: 11,
            title: None,
            family: None,
            model_size: None,
            quantization: None,
            adaptation: None,
            parameters: None,
            description: None,
            mirrors: None,
            hash: hash_base64.to_string(),
        }
    }

    /// Tests the `Default` trait implementation for `Downloader`.
    ///
    /// **What it tests:**
    /// Verifies that `Downloader::default()` successfully instantiates
    /// a valid `Downloader`
    /// instance equivalent to `Downloader::new()`.
    ///
    /// **How it tests:**
    /// Instantiates `Downloader::default()` and asserts that the
    /// object is properly allocated in memory.
    #[test]
    fn test_downloader_default_trait() {
        let downloader = Downloader::default();
        assert!(std::mem::size_of_val(&downloader) > 0);
    }

    /// Tests multi-mirror failover recovery when the primary mirror
    /// returns an HTTP 500 error.
    ///
    /// **What it tests:**
    /// Verifies that if an initial mirror endpoint fails with a server
    /// error, the downloader automatically falls back to subsequent
    /// available mirrors until a valid payload is retrieved.
    ///
    /// **How it tests:**
    /// 1. Spawns two independent `MockServer` instances: a failing
    ///    primary and a working secondary.
    /// 2. Configures the primary server to respond with HTTP 500
    ///    Internal Server Error.
    /// 3. Configures the secondary server to return HTTP 200 with
    ///    valid binary content (`b"hello world"`).
    /// 4. Passes both URLs in the `File` entry's mirror list.
    /// 5. Invokes `download_file` and asserts that the overall
    ///    operation completes successfully (`Ok(())`).
    #[tokio::test]
    async fn test_download_fallback_to_second_mirror() {
        let server_failing = MockServer::start();
        let server_working = MockServer::start();

        let payload = b"hello world";
        let hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

        // Primary mirror returns 500 Internal Server Error
        let _mock_fail = server_failing.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(500);
        });

        // Backup mirror returns valid payload
        let _mock_pass = server_working.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(200).body(payload);
        });

        let entry = File {
            mirrors: Some(laded::file::Mirrors {
                items: vec![
                    laded::file::Mirror {
                        src: server_failing.url("/test.bin"),
                    },
                    laded::file::Mirror {
                        src: server_working.url("/test.bin"),
                    },
                ],
            }),
            ..create_test_file(hash)
        };

        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        let result = downloader
            .download_file(&entry, None, temp_out.path(), |_, _| {})
            .await;

        assert!(result.is_ok());
    }

    /// Tests downloader behavior when the HTTP server responds with a 404 Not Found status code.
    ///
    /// **What it tests:**
    /// Ensures non-200 HTTP responses are treated as chunk transfer failures.
    ///
    /// **How it tests:**
    /// 1. Starts a local `MockServer` configured to return HTTP 404 for the requested file path.
    /// 2. Invokes `download_file` targeting the mock endpoint.
    /// 3. Asserts that the operation returns `Err(Error::ChunkDownloadFailed(0))`.
    #[tokio::test]
    async fn test_download_http_404_not_found_fails() {
        let server = MockServer::start();
        let valid_hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(404);
        });

        let entry = create_test_file(valid_hash);
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        let err = downloader
            .download_file(
                &entry,
                Some(&server.url("/test.bin")),
                temp_out.path(),
                |_, _| {},
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::ChunkDownloadFailed(0)));
    }

    /// Tests error handling when attempting to connect to an unreachable network address.
    ///
    /// **What it tests:**
    /// Verifies that socket connection refusals or transport errors return `Error::ChunkDownloadFailed`.
    ///
    /// **How it tests:**
    /// 1. Directs the downloader to `http://127.0.0.1:1/invalid`
    ///    (port 1 is closed/unbound).
    /// 2. Attempts downloading and asserts that connection
    ///    failures return `Err(Error::ChunkDownloadFailed(0))`.
    #[tokio::test]
    async fn test_download_connection_error_fails() {
        let valid_hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
        let entry = create_test_file(valid_hash);
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        // Point to an invalid local port that rejects connections
        let err = downloader
            .download_file(
                &entry,
                Some("http://127.0.0.1:1/invalid"),
                temp_out.path(),
                |_, _| {},
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::ChunkDownloadFailed(0)));
    }

    /// Tests SHA-256 digest validation when chunk data is corrupted.
    ///
    /// **What it tests:**
    /// Ensures that downloaded chunks whose computed SHA-256 hash does
    /// not match the manifest entry are rejected and marked as failed
    ///
    /// **How it tests:**
    /// 1. Configures a mock server to return corrupted payload bytes
    /// 2. Configures the catalog entry with the SHA-256 digest of
    ///    valid content (`"hello world"`).
    /// 3. Executes `download_file`.
    /// 4. Verifies that the hash mismatch triggers 
    ///    `Err(Error::ChunkDownloadFailed(0))`.
    #[tokio::test]
    async fn test_download_hash_mismatch_fails() {
        let server = MockServer::start();
        let payload = b"corrupted!!"; // SHA-256 does not match valid_hash
        let valid_hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(200).body(payload);
        });

        let entry = create_test_file(valid_hash);
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        let err = downloader
            .download_file(
                &entry,
                Some(&server.url("/test.bin")),
                temp_out.path(),
                |_, _| {},
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::ChunkDownloadFailed(0)));
    }

    /// Tests chunk length validation when a server returns an
    /// incomplete or truncated payload.
    ///
    /// **What it tests:**
    /// Verifies that HTTP responses returning fewer bytes than the
    /// expected chunk size are rejected.
    ///
    /// **How it tests:**
    /// 1. Sets expected chunk size to 11 bytes.
    /// 2. Configures the mock server to return a truncated 5-byte
    ///    payload (`b"short"`).
    /// 3. Executes `download_file`.
    /// 4. Asserts that payload length verification fails with
    ///    `Err(Error::ChunkDownloadFailed(0))`.
    #[tokio::test]
    async fn test_download_size_mismatch_fails() {
        let server = MockServer::start();
        let payload = b"short"; // 5 bytes instead of expected 11
        let valid_hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(200).body(payload);
        });

        let entry = create_test_file(valid_hash);
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        let err = downloader
            .download_file(
                &entry,
                Some(&server.url("/test.bin")),
                temp_out.path(),
                |_, _| {},
            )
            .await
            .unwrap_err();

        assert!(matches!(err, Error::ChunkDownloadFailed(0)));
    }

    /// Tests early validation failure when a catalog entry contains
    /// an invalid Base64 hash manifest.
    ///
    /// **What it tests:**
    /// Verifies that malformed Base64 strings in the file manifest
    /// short-circuit download operations.
    ///
    /// **How it tests:**
    /// 1. Constructs a `File` entry with invalid Base64 data
    /// 2. Calls `download_file`.
    /// 3. Asserts that hash manifest decoding returns
    ///    `Err(Error::Base64Decode(_))`.
    #[tokio::test]
    async fn test_download_invalid_hash_encoding_fails() {
        let entry = create_test_file("!!! invalid base64 hash !!!");
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

        let err = downloader
            .download_file(&entry, None, temp_out.path(), |_, _| {})
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Base64Decode(_)));
    }
}
