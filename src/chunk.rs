/*  src/chunk.rs  Representation, hashing, and verification of chunks. 
 *
 *  Copyright 2026 Emerge Cooperative
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
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};

/// Represents a discrete chunk of a catalog file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Zero-based chunk index
    pub index: usize,
    /// Byte offset where this chunk begins in the file
    pub offset: u64,
    /// Expected byte size of this chunk
    pub size: u64,
    /// Expected SHA-256 digest (32 bytes)
    pub expected_hash: [u8; 32],
}

impl Chunk {
    /// Creates a new `Chunk` descriptor.
    pub fn new(
        index: usize,
        offset: u64,
        size: u64,
        expected_hash: [u8; 32],
    ) -> Self {
        Self {
            index,
            offset,
            size,
            expected_hash,
        }
    }

    /// Formats the HTTP `Range` header value for downloading one chunk
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::chunk::Chunk;
    ///
    /// let chunk = Chunk::new(0, 0, 1024, [0u8; 32]);
    /// assert_eq!(chunk.range_header_value(), "bytes=0-1023");
    /// ```
    pub fn range_header_value(&self) -> String {
        format!("bytes={}-{}", self.offset, self.offset + self.size - 1)
    }

    /// Computes the SHA-256 digest of a byte slice payload.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::chunk::Chunk;
    ///
    /// let hash = Chunk::compute_hash(b"hello world");
    /// assert_eq!(hash.len(), 32);
    /// ```
    pub fn compute_hash(payload: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        hasher.finalize().into()
    }

    /// Decodes base64 strings into 32-byte SHA-256 digests.
    ///
    /// Accepts any combination of spaces, tabs, standard newlines (`\n`),
    /// Windows carriage returns (`\r\n`), or consecutive empty lines.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::chunk::Chunk;
    ///
    /// // Handles mixed whitespace, newlines, tabs, and spaces
    /// let base64_str = concat!(" \r\nuU0nuZNNPgilLlLX2n2r+sSE7"
    ///                          "+N6U4DukIj3rOLvzek=\n \t \n\n");
    /// let hashes = Chunk::decode_base64_hashes(base64_str).unwrap();
    ///
    /// assert_eq!(hashes.len(), 1);
    /// assert_eq!(hashes[0], Chunk::compute_hash(b"hello world"));
    /// ```
    ///
    /// # Errors
    /// Returns `Error::Base64Decode` if decoding fails, or
    /// `Error::InvalidHashSize` if a decoded digest isn't 32 bytes long
    pub fn decode_base64_hashes(
        raw_hash_str: &str,
    ) -> Result<Vec<[u8; 32]>, Error> {
        let mut digests = Vec::new();
        for token in raw_hash_str.split_whitespace() {
            let bytes = BASE64
                .decode(token)
                .map_err(|e| Error::Base64Decode(e.to_string()))?;

            if bytes.len() != 32 {
                return Err(Error::InvalidHashSize(bytes.len()));
            }

            let mut hash_arr = [0u8; 32];
            hash_arr.copy_from_slice(&bytes);
            digests.push(hash_arr);
        }
        Ok(digests)
    }

    /// Verifies that a byte payload matches the SHA-256 hash.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::chunk::Chunk;
    ///
    /// let payload = b"hello world";
    /// let hash = Chunk::compute_hash(payload);
    /// let chunk = Chunk::new(0, 0, 11, hash);
    ///
    /// assert!(chunk.verify(payload));
    /// assert!(!chunk.verify(b"corrupted payload"));
    /// ```
    ///
    /// Successful match using a known SHA-256 hex digest (`"Testing\n"`):
    ///
    /// ```rust
    /// use laded::chunk::Chunk;
    ///
    /// // String generated via: echo "Testing" | sha256sum
    /// let payload = b"Testing\n";
    /// let hash: [u8; 32] = [
    ///     0x41, 0x92, 0x0b, 0x34, 0x8e, 0x0c, 0x6f, 0xf2,
    ///     0xef, 0x9b, 0x7e, 0x3e, 0xe9, 0x30, 0x87, 0x26,
    ///     0xaa, 0x52, 0x50, 0xfa, 0x71, 0x78, 0x83, 0xe0,
    ///     0x73, 0xff, 0x6a, 0x93, 0x6a, 0x93, 0x25, 0xa4,
    /// ];
    ///
    /// let chunk = Chunk::new(0, 0, payload.len() as u64, hash);
    /// assert!(chunk.verify(payload));
    /// ```
    ///

    pub fn verify(&self, payload: &[u8]) -> bool {
        if payload.len() as u64 != self.size {
            return false;
        }
        Self::compute_hash(payload) == self.expected_hash
    }
}
