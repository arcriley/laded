/*  src/builder.rs  Catalog Builder generation.
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

use crate::catalog::Catalog;
use crate::chunk::Chunk;
use crate::directory::Directory;
use crate::error::Error;
use crate::file::{File, Mirror, Mirrors};
use crate::package::Package;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;

pub struct Builder {
    version: String,
    packages: Vec<Package>,
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
    /// assert!(xml.contains("version=\"1.2\""));
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
    /// assert!(xml.contains("version=\"1.2\""));
    /// ```
    pub fn new() -> Self {
        Self {
            version: "1.2".to_string(),
            packages: Vec::new(),
        }
    }

    /// Adds a fully-constructed package directly to the catalog builder.
    pub fn add_package(&mut self, package: Package) -> &mut Self {
        self.packages.push(package);
        self
    }

    /// Helper method to create a `File` struct from a local disk path and list of mirror URLs.
    pub fn create_file_from_disk(
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
    ) -> Result<File, Error> {
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

        let mirror_items: Vec<Mirror> = mirrors.into_iter().map(|src| Mirror { src }).collect();
        let mirrors_struct = if mirror_items.is_empty() {
            None
        } else {
            Some(Mirrors { items: mirror_items })
        };

        Ok(File {
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
        })
    }

    /// Reads a file from disk, hashes its contents in chunks, and adds it as a single-file
    /// package at the root level of the catalog.
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
    ///         vec!["[https://example.com/file.bin](https://example.com/file.bin)".to_string()],
    ///     )
    ///     .unwrap();
    ///
    /// let xml = builder.build().unwrap();
    /// assert!(xml.contains("[https://example.com/file.bin](https://example.com/file.bin)"));
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
        let file_entry = Self::create_file_from_disk(
            path,
            chunk_size,
            title.clone(),
            family.clone(),
            model_size.clone(),
            quantization.clone(),
            adaptation.clone(),
            parameters.clone(),
            description.clone(),
            mirrors,
        )?;

        let package_title = title.unwrap_or_else(|| file_entry.name.clone());

        let package = Package {
            title: package_title,
            description,
            family,
            model_size,
            quantization,
            adaptation,
            parameters,
            files: vec![file_entry],
            directories: vec![],
        };

        self.packages.push(package);
        Ok(self)
    }

    /// Recursively builds a `Directory` node from a directory on disk.
    pub fn add_directory_from_disk(
        dir_path: &Path,
        chunk_size: u32,
        base_mirror_url: &str,
    ) -> Result<Directory, Error> {
        let dir_name = dir_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::XmlParse("Invalid directory name".to_string()))?
            .to_string();

        let mut files = Vec::new();
        let mut directories = Vec::new();

        for entry in std::fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let sub_dir = Self::add_directory_from_disk(
                    &path,
                    chunk_size,
                    &format!("{}/{}", base_mirror_url, path.file_name().unwrap().to_string_lossy()),
                )?;
                directories.push(sub_dir);
            } else if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let mirror_url = format!("{}/{}", base_mirror_url, file_name);

                let file_entry = Self::create_file_from_disk(
                    &path,
                    chunk_size,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    vec![mirror_url],
                )?;
                files.push(file_entry);
            }
        }

        Ok(Directory {
            name: dir_name,
            files,
            directories,
        })
    }

    /// Serializes accumulated catalog package entries into XML format.
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
        let catalog = Catalog {
            version: self.version.clone(),
            packages: self.packages.clone(),
        };

        quick_xml::se::to_string(&catalog)
            .map_err(|e| Error::XmlParse(e.to_string()))
    }
}
