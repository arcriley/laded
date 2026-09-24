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
    use laded::file::{File, Mirror, Mirrors};
    use tempfile::NamedTempFile;

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

    #[test]
    fn test_downloader_default_trait() {
        let downloader = Downloader::default();
        assert!(std::mem::size_of_val(&downloader) > 0);
    }

    #[tokio::test]
    async fn test_download_fallback_to_second_mirror() {
        let server_failing = MockServer::start();
        let server_working = MockServer::start();

        let payload = b"hello world";
        let hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";

        let _mock_fail = server_failing.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(500);
        });

        let _mock_pass = server_working.mock(|when, then| {
            when.method(GET).path("/test.bin");
            then.status(200).body(payload);
        });

        let entry = File {
            mirrors: Some(Mirrors {
                items: vec![
                    Mirror {
                        src: server_failing.url("/test.bin"),
                    },
                    Mirror {
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

    #[tokio::test]
    async fn test_download_connection_error_fails() {
        let valid_hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
        let entry = create_test_file(valid_hash);
        let temp_out = NamedTempFile::new().unwrap();
        let downloader = Downloader::new();

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

    #[tokio::test]
    async fn test_download_hash_mismatch_fails() {
        let server = MockServer::start();
        let payload = b"corrupted!!";
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

    #[tokio::test]
    async fn test_download_size_mismatch_fails() {
        let server = MockServer::start();
        let payload = b"short";
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
