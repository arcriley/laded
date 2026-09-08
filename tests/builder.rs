/*  tests/builder.rs  Error handling and edge case integration tests
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
use laded::error::Error;
use std::path::Path;

/// Tests default initialization via `Builder::default`.
///
/// **What it tests:**
/// Ensures `Builder::default()` constructs a valid instance equivalent
/// to `Builder::new()`.
///
/// **How it tests:**
/// Calls `Builder::default()` and asserts it builds an empty catalog
/// XML document.
#[test]
fn test_default() {
    let builder = Builder::default();
    let xml = builder.build().unwrap();
    assert!(xml.contains("<catalog"));
}

/// Tests `add_file_from_disk` with a nonexistent file path.
///
/// **What it tests:**
/// Verifies `add_file_from_disk` returns `Error::Io` when attempting
/// to open a missing path.
///
/// **How it tests:**
/// Passes a non-existent path to `add_file_from_disk` and checks the
/// error type.
#[test]
fn test_add_file_missing_path() {
    let mut builder = Builder::new();
    let result = builder.add_file_from_disk(
        Path::new("/nonexistent/file/path.bin"),
        1024,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![],
    );

    assert!(matches!(result, Err(Error::Io(_))));
}

/// Tests `add_file_from_disk` when given a path without a valid file name.
///
/// **What it tests:**
/// Verifies `add_file_from_disk` returns `Error::XmlParse` when the
/// target path ends in `..` or root.
///
/// **How it tests:**
/// Passes `Path::new("/")` to `add_file_from_disk` and asserts an
/// invalid filename error is returned.
#[test]
fn test_add_file_invalid_filename() {
    let mut builder = Builder::new();
    let result = builder.add_file_from_disk(
        Path::new("/"),
        1024,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        vec![],
    );

    assert!(matches!(result, Err(Error::XmlParse(_))));
}
