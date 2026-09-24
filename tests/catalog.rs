/*  tests/catalog.rs  Integration tests for complex catalog parsing and downloading.
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

use laded::builder::Builder;
use laded::catalog::Catalog;
use laded::downloader::Downloader;
use laded::package::Package;
use std::fs;
use std::net::SocketAddr;
use tempfile::TempDir;
use tokio::sync::oneshot;

const CHUNK_SIZE: u32 = 64;

struct TestServer {
    pub addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl TestServer {
    pub async fn start(static_dir: &std::path::Path) -> Self {
        let routes = warp::fs::dir(static_dir.to_path_buf());
        let (tx, rx) = oneshot::channel::<()>();

        let (addr, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
                rx.await.ok();
            });

        tokio::spawn(server);

        TestServer {
            addr,
            shutdown_tx: Some(tx),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}/{}", self.addr, path.trim_start_matches('/'))
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[tokio::test]
async fn test_deep_tree_and_edge_cases() {
    let payload_dir = TempDir::new().unwrap();
    let root_path = payload_dir.path();

    // Setup payload directory structure with edge cases
    let root_file1 = root_path.join("root_file.txt");
    let root_file2 = root_path.join("README.md");
    let deep_dir = root_path.join("models").join("sub").join("nested");
    let empty_dir = root_path.join("empty_folder");

    fs::create_dir_all(&deep_dir).unwrap();
    fs::create_dir_all(&empty_dir).unwrap();

    fs::write(&root_file1, b"Root File Payload Data Vector").unwrap();
    fs::write(&root_file2, b"Documentation Markdown Content").unwrap();

    let deep_file = deep_dir.join("weights.bin");
    fs::write(&deep_file, vec![0xAB; 256]).unwrap();

    // Start local server hosting payload files
    let server = TestServer::start(root_path).await;

    // Construct Package 1: Hybrid Package containing root files and a deep directory tree
    let file1_entry = Builder::create_file_from_disk(
        &root_file1,
        CHUNK_SIZE,
        Some("Root File".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        vec![server.url("root_file.txt")],
    )
    .unwrap();

    let file2_entry = Builder::create_file_from_disk(
        &root_file2,
        CHUNK_SIZE,
        Some("README".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        vec![server.url("README.md")],
    )
    .unwrap();

    let deep_directory =
        Builder::add_directory_from_disk(&root_path.join("models"), CHUNK_SIZE, &server.url("models"))
            .unwrap();

    let package1 = Package {
        title: "Hybrid Root & Deep Package".to_string(),
        description: Some("Tests multiple root files and deep directory trees".to_string()),
        family: Some("TestFamily".to_string()),
        model_size: None,
        quantization: Some("Q4_0".to_string()),
        adaptation: None,
        parameters: None,
        files: vec![file1_entry, file2_entry],
        directories: vec![deep_directory],
    };

    // Construct Package 2: Directory-Only Package
    let dir_only = Builder::add_directory_from_disk(
        &root_path.join("models"),
        CHUNK_SIZE,
        &server.url("models"),
    )
    .unwrap();

    let package2 = Package {
        title: "Directory Only Package".to_string(),
        description: Some("Edge case: package with no root files".to_string()),
        family: None,
        model_size: None,
        quantization: None,
        adaptation: None,
        parameters: None,
        files: vec![],
        directories: vec![dir_only],
    };

    // Build Catalog XML
    let mut builder = Builder::new();
    builder.add_package(package1);
    builder.add_package(package2);

    let catalog_xml = builder.build().unwrap();
    fs::write(root_path.join("catalog.lading"), &catalog_xml).unwrap();

    // Verify loading catalog over HTTP
    let catalog_url = server.url("catalog.lading");
    let loaded_catalog = Catalog::fetch(&catalog_url).await.unwrap();

    assert_eq!(loaded_catalog.packages.len(), 2);

    // Verify Package 1 structures
    let pkg1 = &loaded_catalog.packages[0];
    assert_eq!(pkg1.files.len(), 2);
    assert_eq!(pkg1.directories.len(), 1);

    let all_files_pkg1 = pkg1.all_files();
    assert_eq!(all_files_pkg1.len(), 3); // 2 root files + 1 deep file

    // Download Package 1 to temporary target directory
    let download_target = TempDir::new().unwrap();
    let downloader = Downloader::new();

    downloader
        .download_package(pkg1, download_target.path(), |_read, _total| {})
        .await
        .unwrap();

    // Assert downloaded file layout on disk
    assert!(download_target.path().join("root_file.txt").exists());
    assert!(download_target.path().join("README.md").exists());

    let downloaded_deep_file = download_target
        .path()
        .join("models")
        .join("sub")
        .join("nested")
        .join("weights.bin");
    assert!(downloaded_deep_file.exists());
    assert_eq!(fs::read(downloaded_deep_file).unwrap(), vec![0xAB; 256]);
}
