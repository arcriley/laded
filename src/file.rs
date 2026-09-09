/*  src/file.rs  Catalog file entry descriptors and chunk mapping.
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

use crate::chunk::Chunk;
use crate::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    #[serde(rename = "@src")]
    pub src: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirrors {
    #[serde(rename = "mirror", default)]
    pub items: Vec<Mirror>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@file_size")]
    pub file_size: u64,

    #[serde(rename = "@chunk_size")]
    pub chunk_size: u32,

    #[serde(rename = "@title")]
    pub title: Option<String>,

    #[serde(rename = "@family")]
    pub family: Option<String>,

    #[serde(rename = "@model_size")]
    pub model_size: Option<String>,

    #[serde(rename = "@quantization")]
    pub quantization: Option<String>,

    #[serde(rename = "@adaptation")]
    pub adaptation: Option<String>,

    #[serde(rename = "@parameters")]
    pub parameters: Option<String>,

    #[serde(rename = "@description")]
    pub description: Option<String>,

    #[serde(rename = "mirrors")]
    pub mirrors: Option<Mirrors>,

    #[serde(rename = "hash")]
    pub hash: String,
}

impl File {
    /// Returns a list of mirror URLs defined for this file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::file::{File, Mirror, Mirrors};
    ///
    /// let file = File {
    ///     name: "test.bin".to_string(),
    ///     file_size: 100,
    ///     chunk_size: 50,
    ///     title: None,
    ///     family: None,
    ///     model_size: None,
    ///     quantization: None,
    ///     adaptation: None,
    ///     parameters: None,
    ///     description: None,
    ///     mirrors: Some(Mirrors {
    ///         items: vec![Mirror {
    ///             src: "https://example.com/test.bin".to_string(),
    ///         }],
    ///     }),
    ///     hash: String::new(),
    /// };
    ///
    /// assert_eq!(
    ///     file.mirror_urls(),
    ///     vec!["https://example.com/test.bin".to_string()]
    /// );
    /// ```
    pub fn mirror_urls(&self) -> Vec<String> {
        self.mirrors
            .as_ref()
            .map(|m| m.items.iter().map(|item| item.src.clone()).collect())
            .unwrap_or_default()
    }

    /// Calculates the byte size of the trailing chunk.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::file::File;
    ///
    /// let file = File {
    ///     name: "test.bin".to_string(),
    ///     file_size: 105,
    ///     chunk_size: 50,
    ///     title: None,
    ///     family: None,
    ///     model_size: None,
    ///     quantization: None,
    ///     adaptation: None,
    ///     parameters: None,
    ///     description: None,
    ///     mirrors: None,
    ///     hash: String::new(),
    /// };
    ///
    /// assert_eq!(file.last_chunk_size(), 5);
    /// ```
    pub fn last_chunk_size(&self) -> u32 {
        let remainder = (self.file_size % self.chunk_size as u64) as u32;
        if remainder == 0 {
            self.chunk_size
        } else {
            remainder
        }
    }

    /// Decodes base64 chunk hashes embedded inside the file entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::file::File;
    ///
    /// let hash_str = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
    /// let file = File {
    ///     name: "test.bin".to_string(),
    ///     file_size: 11,
    ///     chunk_size: 11,
    ///     title: None,
    ///     family: None,
    ///     model_size: None,
    ///     quantization: None,
    ///     adaptation: None,
    ///     parameters: None,
    ///     description: None,
    ///     mirrors: None,
    ///     hash: hash_str.to_string(),
    /// };
    ///
    /// let hashes = file.decode_hashes().unwrap();
    /// assert_eq!(hashes.len(), 1);
    /// ```
    pub fn decode_hashes(&self) -> Result<Vec<[u8; 32]>, Error> {
        Chunk::decode_base64_hashes(&self.hash)
    }

    /// Constructs a list of `Chunk` descriptors representing all chunks
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::file::File;
    ///
    /// let hash_str = "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=\n";
    /// let file = File {
    ///     name: "test.bin".to_string(),
    ///     file_size: 11,
    ///     chunk_size: 11,
    ///     title: None,
    ///     family: None,
    ///     model_size: None,
    ///     quantization: None,
    ///     adaptation: None,
    ///     parameters: None,
    ///     description: None,
    ///     mirrors: None,
    ///     hash: hash_str.to_string(),
    /// };
    ///
    /// let chunks = file.chunks().unwrap();
    /// assert_eq!(chunks.len(), 1);
    /// assert_eq!(chunks[0].size, 11);
    /// assert_eq!(chunks[0].offset, 0);
    /// ```
    pub fn chunks(&self) -> Result<Vec<Chunk>, Error> {
        let hashes = self.decode_hashes()?;
        let chunk_size = self.chunk_size as u64;
        let total = hashes.len();
        let mut chunks = Vec::with_capacity(total);

        for (i, expected_hash) in hashes.into_iter().enumerate() {
            let offset = i as u64 * chunk_size;
            let size = if i == total - 1 {
                self.last_chunk_size() as u64
            } else {
                chunk_size
            };

            chunks.push(Chunk::new(i, offset, size, expected_hash));
        }

        Ok(chunks)
    }
}
