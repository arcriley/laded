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

/// Tests the `laded pack` command when outputting to stdout.
///
/// **What it tests:**
/// Verify that the CLI can accept an input file path and metadata
/// flags (such as `--title`), build a valid catalog XML structure,
/// and write the resulting XML directly to standard output.
///
/// **How it tests:**
/// 1. Creates a temporary dummy input file with sample content.
/// 2. Invokes the `laded pack` CLI binary
/// 3. Asserts the binary exits successfully (exit code 0)
/// 4. Verifies standard output contains the root `<catalog`
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

/// Tests packing a file to an XML catalog on disk and subsequently
/// inspecting it via `laded list`.
///
/// **What it tests:**
/// Verify tcatalog creation and metadata inspection CLI workflows:
/// - `pack` properly serializes file attributes, model metadata,
///   and download endpoints to an output XML file.
/// - `list` reads and parses the generated catalog XML, outputting
///   human-readable summaries to stdout.
///
/// **How it tests:**
/// 1. Prepares a temporary input file and a target destination file
///    for the catalog XML.
/// 2. Runs `laded pack` specifying along with metadata fields.
/// 3. Asserts that the catalog file creation completes successfully.
/// 4. Executes `laded list --catalog <catalog_path>`.
/// 5. Asserts the command succeeds and stdout contains all parsed
///    catalog attributes (Version, Family, Quantization, Mirror URL).
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

/// Tests the full `laded get` file download and SHA-256 chunk
/// verification CLI pipeline.
///
/// **What it tests:**
/// Verifies that `laded get` can parse a catalog XML manifest,
/// connect to configured mirror URLs, fetch range chunks,
/// compute and verify chunk hashes, and write the reconstructed
/// payload to disk.
///
/// **How it tests:**
/// 1. Starts a local HTTP mock server using `httpmock` configured
///    to serve a payload on GET request.
/// 2. Generates a catalog XML pointing to the mock server's local mirror URL.
/// 3. Executes `laded get --source <catalog_path> --output <destination_path>`.
/// 4. Asserts exit code 0 and verifies progress output in stderr ("Download completed successfully.").
/// 5. Compares the downloaded file on disk byte-for-byte against the original source payload.
#[test]
fn test_cli_get_download() {
    let server = MockServer::start();
    let payload = b"cli download payload";

    // Set up local mock HTTP endpoint to serve range request
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/cli_download.bin");
        then.status(200).body(payload);
    });

    let mock_url = server.url("/cli_download.bin");

    let input_file = NamedTempFile::new().unwrap();
    let catalog_file = NamedTempFile::new().unwrap();
    let output_file = NamedTempFile::new().unwrap();

    fs::write(input_file.path(), payload).unwrap();

    // Pack the file pointing to the mock HTTP server mirror endpoint
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

    // Perform download via 'get' sub-command
    let mut get_cmd = Command::cargo_bin("laded").unwrap();
    get_cmd
        .arg("get")
        .arg("--source")
        .arg(catalog_file.path())
        .arg("--output")
        .arg(output_file.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Download completed successfully."));

    // Ensure reconstructed output file matches target payload
    assert_eq!(fs::read(output_file.path()).unwrap(), payload);
}

/// Tests CLI error handling when requesting a file name that does not exist in the catalog.
///
/// **What it tests:**
/// Verifies that passing an explicit `--name` argument that does not correspond to any `<file>` entry
/// inside the parsed catalog causes `laded get` to fail gracefully with an appropriate error message.
///
/// **How it tests:**
/// 1. Generates a valid catalog file indexing an entry named after the temporary file.
/// 2. Executes `laded get --source <catalog_path> --name nonexistent.bin`.
/// 3. Asserts that the command fails (non-zero exit status).
/// 4. Checks stderr to verify the expected error context ("File 'nonexistent.bin' not found in catalog") is returned.
#[test]
fn test_cli_get_file_not_found_in_catalog() {
    let input_file = NamedTempFile::new().unwrap();
    let catalog_file = NamedTempFile::new().unwrap();

    fs::write(input_file.path(), b"data").unwrap();

    // Pack a valid catalog
    Command::cargo_bin("laded")
        .unwrap()
        .arg("pack")
        .arg("--input")
        .arg(input_file.path())
        .arg("--output")
        .arg(catalog_file.path())
        .assert()
        .success();

    // Request a file entry name that does not exist in the catalog
    Command::cargo_bin("laded")
        .unwrap()
        .arg("get")
        .arg("--source")
        .arg(catalog_file.path())
        .arg("--name")
        .arg("nonexistent.bin")
        .assert()
        .failure()
        .stderr(predicate::str::contains("File 'nonexistent.bin' not found in catalog"));
}
