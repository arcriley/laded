/*  src/builder.rs  Catalog Builder  generation.
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
use crate::file::{File, Mirror, Mirrors};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;

pub struct Builder {
    version: String,
    files: Vec<File>,
}

impl Default for Builder {
    /// Creates a default `Builder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::builder::Builder;
    ///
    /// let builder = Builder::default();
    /// let xml = builder.build().unwrap();
    /// assert!(xml.contains("version=\"1.0\""));
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new `Builder` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::builder::Builder;
    ///
    /// let builder = Builder::new();
    /// let xml = builder.build().unwrap();
    /// assert!(xml.contains("version=\"1.0\""));
    /// ```
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            files: Vec::new(),
        }
    }

    /// Reads a file from disk, hashes its contents in chunks, and adds
    /// a file entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::builder::Builder;
    /// use tempfile::NamedTempFile;
    /// use std::io::Write;
    ///
    /// let mut dummy_file = NamedTempFile::new().unwrap();
    /// dummy_file.write_all(b"hello world payload").unwrap();
    ///
    /// let mut builder = Builder::new();
    /// builder
    ///     .add_file_from_disk(
    ///         dummy_file.path(),
    ///         10,
    ///         Some("Sample File".to_string()),
    ///         None,
    ///         None,
    ///         None,
    ///         None,
    ///         None,
    ///         None,
    ///         vec!["https://example.com/file.bin".to_string()],
    ///     )
    ///     .unwrap();
    ///
    /// let xml = builder.build().unwrap();
    /// assert!(xml.contains("https://example.com/file.bin"));
    /// assert!(xml.contains("title=\"Sample File\""));
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn add_file_from_disk(
        &mut self,
        path: &Path,
        chunk_size: u32,
        title: Option<String>,
        family: Option<String>,
        model_size: Option<String>,
        quantization: Option<String>,
        adaptation: Option<String>,
        parameters: Option<String>,
        description: Option<String>,
        mirrors: Vec<String>,
    ) -> Result<&mut Self, Error> {
        let mut file = StdFile::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::XmlParse("Invalid filename".to_string()))?
            .to_string();

        let mut hash_lines = Vec::new();
        let mut buffer = vec![0u8; chunk_size as usize];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let digest = Chunk::compute_hash(&buffer[..bytes_read]);
            let base64_digest = BASE64.encode(digest);
            hash_lines.push(base64_digest);
        }

        let formatted_hash = format!("\n{}\n", hash_lines.join("\n"));

        let mirror_items: Vec<Mirror> = mirrors.into_iter().
            map(|src| Mirror { src }).collect();
        let mirrors_struct = if mirror_items.is_empty() {
            None
        } else {
            Some(Mirrors { items: mirror_items })
        };

        let file_entry = File {
            name: file_name,
            file_size,
            chunk_size,
            title,
            family,
            model_size,
            quantization,
            adaptation,
            parameters,
            description,
            mirrors: mirrors_struct,
            hash: formatted_hash,
        };

        self.files.push(file_entry);
        Ok(self)
    }

    /// Serializes accumulated catalog file entries into XML format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::builder::Builder;
    ///
    /// let builder = Builder::new();
    /// let xml = builder.build().unwrap();
    /// assert!(xml.starts_with("<catalog"));
    /// ```
    pub fn build(&self) -> Result<String, Error> {
        let catalog = crate::catalog::Catalog {
            version: self.version.clone(),
            files: self.files.clone(),
        };

        quick_xml::se::to_string(&catalog)
            .map_err(|e| Error::XmlParse(e.to_string()))
    }
}
