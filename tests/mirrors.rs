/*  tests/mirrors.rs  Mirror failover and latency test suite
 *
 *  Copyright 2026 Emerge Cooperative
 */

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use warp::Filter;

use laded::downloader::Downloader;
use laded::file::{File, Mirror, Mirrors};
use tempfile::NamedTempFile;

/// Helper mock server for mirror tests using warp.
pub struct MockServer {
    addr: SocketAddr,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl MockServer {
    pub async fn start<R>(route: R) -> Self
    where
        R: Filter + Clone + Send + Sync + 'static,
        R::Extract: warp::Reply,
    {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let (addr, server) = warp::serve(route).bind_ephemeral(([127, 0, 0, 1], 0));

        let graceful_server = async move {
            tokio::select! {
                _ = server => {},
                _ = rx => {},
            }
        };

        tokio::spawn(graceful_server);

        Self {
            addr,
            _shutdown_tx: tx,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}/{}", self.addr, path.trim_start_matches('/'))
    }
}

fn create_test_file(hash_base64: &str, mirrors: Vec<String>) -> File {
    File {
        name: "data.bin".to_string(),
        file_size: 11,
        chunk_size: 11,
        title: None,
        family: None,
        model_size: None,
        quantization: None,
        adaptation: None,
        parameters: None,
        description: None,
        mirrors: Some(Mirrors {
            items: mirrors.into_iter().map(|src| Mirror { src }).collect(),
        }),
        hash: hash_base64.to_string(),
    }
}

#[tokio::test]
async fn test_slow_mirror_fallback() {
    let payload = Arc::new(b"hello world".to_vec());

    let slow_data = payload.clone();
    let slow_route = warp::path("data.bin").and_then(move || {
        let _data = slow_data.clone();
        async move {
            sleep(Duration::from_secs(2)).await;
            Ok::<_, warp::Rejection>((*_data).clone())
        }
    });

    let fast_data = payload.clone();
    let fast_route = warp::path("data.bin").map(move || (*fast_data).clone());

    let slow_server = MockServer::start(slow_route).await;
    let fast_server = MockServer::start(fast_route).await;

    // Base64 SHA-256 for b"hello world"
    let hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
    let file_entry = create_test_file(
        hash,
        vec![slow_server.url("data.bin"), fast_server.url("data.bin")],
    );

    let temp_out = NamedTempFile::new().unwrap();
    let downloader = Downloader::new();

    let result = downloader
        .download_file(&file_entry, None, temp_out.path(), |_, _| {})
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_corrupted_mirror_failover() {
    let corrupted_payload = Arc::new(vec![0u8; 11]);
    let valid_payload = Arc::new(b"hello world".to_vec());

    let bad_data = corrupted_payload.clone();
    let bad_route = warp::path("chunk.bin").map(move || (*bad_data).clone());

    let good_data = valid_payload.clone();
    let good_route = warp::path("chunk.bin").map(move || (*good_data).clone());

    let bad_server = MockServer::start(bad_route).await;
    let good_server = MockServer::start(good_route).await;

    // Base64 SHA-256 for b"hello world"
    let hash = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
    let file_entry = create_test_file(
        hash,
        vec![bad_server.url("chunk.bin"), good_server.url("chunk.bin")],
    );

    let temp_out = NamedTempFile::new().unwrap();
    let downloader = Downloader::new();

    let result = downloader
        .download_file(&file_entry, None, temp_out.path(), |_, _| {})
        .await;

    assert!(result.is_ok());
}
