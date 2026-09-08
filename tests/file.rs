/*  tests/file.rs  Error handling and edge case integration tests
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

use laded::error::Error;
use laded::file::File;

/// Helper function to construct a base `File` struct
fn create_test_file(
    file_size: u64,
    chunk_size: u32,
    hash: &str) -> File {
    File {
        name: "test.bin".to_string(),
        file_size,
        chunk_size,
        title: None,
        family: None,
        model_size: None,
        quantization: None,
        adaptation: None,
        parameters: None,
        description: None,
        mirrors: None,
        hash: hash.to_string(),
    }
}

/// Tests `File::mirror_urls` when no mirrors are specified
///
/// **What it tests:**
/// Verifies that `mirror_urls()` returns an empty `Vec` rather than
/// panicking when `mirrors` is `None`.
///
/// **How it tests:**
/// Calls `mirror_urls()` on a `File` with `mirrors: None` and asserts
/// `is_empty()`.
#[test]
fn test_file_mirror_urls_none() {
    let file = create_test_file(100, 50, "");
    assert!(file.mirror_urls().is_empty());
}

/// Tests `File::chunks` error propagation when hash decoding fails.
///
/// **What it tests:**
/// Ensures `File::chunks()` correctly bubble up Base64 and invalid
/// hash size errors returned by `decode_hashes()`.
///
/// **How it tests:**
/// Passes invalid Base64 in `file.hash` and asserts `file.chunks()`
/// returns `Err(Error::Base64Decode(_))`.
#[test]
fn test_file_chunks_propagates_decode_error() {
    let file = create_test_file(100, 50, "invalid_base64!!!");
    let result = file.chunks();
    assert!(matches!(result, Err(Error::Base64Decode(_))));
}

/// Tests boundary multi-chunk layout calculations in `File::chunks`.
///
/// **What it tests:**
/// Verifies that `File::chunks()` correctly computes chunk offsets,
/// non-final sizes, and last chunk remainder sizes.
///
/// **How it tests:**
/// 1. Constructs a file of 105 bytes split into 50-byte chunk sizes
/// (yielding chunks of sizes 50, 50, and 5).
/// 2. Verifies that `chunks()` returns 3 chunk descriptors with
/// offsets `0`, `50`, and `100`.
#[test]
fn test_file_chunks_multi_chunk_boundary_calculation() {
    // Generate 3 valid 32-byte dummy hashes encoded in Base64
    let hash_1 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [1u8; 32]);
    let hash_2 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [2u8; 32]);
    let hash_3 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [3u8; 32]);
    let multi_hash = format!("{}\n{}\n{}\n", hash_1, hash_2, hash_3);

    let file = create_test_file(105, 50, &multi_hash);
    let chunks = file.chunks().unwrap();

    assert_eq!(chunks.len(), 3);
    
    // Chunk 0: 50 bytes, offset 0
    assert_eq!(chunks[0].offset, 0);
    assert_eq!(chunks[0].size, 50);

    // Chunk 1: 50 bytes, offset 50
    assert_eq!(chunks[1].offset, 50);
    assert_eq!(chunks[1].size, 50);

    // Chunk 2 (last chunk): 5 bytes remainder, offset 100
    assert_eq!(chunks[2].offset, 100);
    assert_eq!(chunks[2].size, 5);
}
