/*  tests/cli.rs  CLI integration tests for the `laded` binary.
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

use assert_cmd::Command;
use httpmock::prelude::*;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_cli_pack_to_stdout() {
    let input_file = NamedTempFile::new().unwrap();
    fs::write(input_file.path(), b"cli test content").unwrap();

    let mut cmd = Command::cargo_bin("laded").unwrap();
    cmd.arg("pack")
        .arg("--input")
        .arg(input_file.path())
        .arg("--title")
        .arg("Test Title")
        .assert()
        .success()
        .stdout(predicate::str::contains("<catalog"))
        .stdout(predicate::str::contains("Test Title"));
}

#[test]
fn test_cli_pack_to_file_and_list() {
    let input_file = NamedTempFile::new().unwrap();
    let catalog_file = NamedTempFile::new().unwrap();
    fs::write(input_file.path(), b"hello catalog").unwrap();

    // Step 1: Pack the input file into an XML catalog on disk
    let mut pack_cmd = Command::cargo_bin("laded").unwrap();
    pack_cmd
        .arg("pack")
        .arg("--input")
        .arg(input_file.path())
        .arg("--output")
        .arg(catalog_file.path())
        .arg("--family")
        .arg("Llama")
        .arg("--quantization")
        .arg("Q4_0")
        .arg("--mirror")
        .arg("http://localhost/dummy.bin")
        .assert()
        .success();

    // Step 2: List and inspect the contents of the newly generated catalog
    let mut list_cmd = Command::cargo_bin("laded").unwrap();
    list_cmd
        .arg("list")
        .arg("--catalog")
        .arg(catalog_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Catalog Version:"))
        .stdout(predicate::str::contains("Family: Llama"))
        .stdout(predicate::str::contains("Quantization: Q4_0"))
        .stdout(predicate::str::contains("http://localhost/dummy.bin"));
}

#[test]
fn test_cli_get_download() {
    let server = MockServer::start();
    let payload = b"cli download payload";

    let _mock = server.mock(|when, then| {
        when.method(GET).path("/cli_download.bin");
        then.status(200).body(payload);
    });

    let mock_url = server.url("/cli_download.bin");

    let input_file = NamedTempFile::new().unwrap();
    let catalog_file = NamedTempFile::new().unwrap();

    // --output expects a directory for `laded get`
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(input_file.path(), payload).unwrap();

    let mut pack_cmd = Command::cargo_bin("laded").unwrap();
    pack_cmd
        .arg("pack")
        .arg("--input")
        .arg(input_file.path())
        .arg("--output")
        .arg(catalog_file.path())
        .arg("--mirror")
        .arg(&mock_url)
        .assert()
        .success();

    let input_filename = input_file
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let mut get_cmd = Command::cargo_bin("laded").unwrap();
    get_cmd
        .arg("get")
        .arg("--source")
        .arg(catalog_file.path())
        .arg("--name")
        .arg(input_filename)
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    let downloaded_file_path = temp_dir.path().join(input_filename);
    assert_eq!(fs::read(&downloaded_file_path).unwrap(), payload);
}

#[test]
fn test_cli_get_file_not_found_in_catalog() {
    let input_file = NamedTempFile::new().unwrap();
    let catalog_file = NamedTempFile::new().unwrap();

    fs::write(input_file.path(), b"data").unwrap();

    Command::cargo_bin("laded")
        .unwrap()
        .arg("pack")
        .arg("--input")
        .arg(input_file.path())
        .arg("--output")
        .arg(catalog_file.path())
        .assert()
        .success();

    Command::cargo_bin("laded")
        .unwrap()
        .arg("get")
        .arg("--source")
        .arg(catalog_file.path())
        .arg("--name")
        .arg("nonexistent.bin")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Entry 'nonexistent.bin' not found in catalog"));
}
