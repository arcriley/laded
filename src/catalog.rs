/*  src/catalog.rs  XML catalog parser and file entry container.
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

use crate::error::Error;
use crate::file::File;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "catalog")]
pub struct Catalog {
    #[serde(rename = "@version")]
    pub version: String,

    #[serde(rename = "file", default)]
    pub files: Vec<File>,
}

impl Catalog {
    /// Asynchronously fetches an XML catalog file from a remote URL.
    ///
    /// # Arguments
    /// - `url`: Public HTTP/HTTPS endpoint pointing to `.lading` file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use httpmock::prelude::*;
    /// use laded::catalog::Catalog;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let server = MockServer::start();
    ///     let xml_payload = r#"<?xml version="1.0" encoding="UTF-8"?>
    /// <catalog version="1.0">
    ///     <file name="test.gguf" file_size="10" chunk_size="10">
    ///         <hash>hash</hash>
    ///     </file>
    /// </catalog>"#;
    ///
    ///     let _mock = server.mock(|when, then| {
    ///         when.method(GET).path("/catalog.lading");
    ///         then.status(200).body(xml_payload);
    ///     });
    ///
    ///     let catalog_url = server.url("/catalog.lading");
    ///     let catalog = Catalog::fetch(&catalog_url).await.unwrap();
    ///     assert_eq!(catalog.files().len(), 1);
    /// }
    /// ```
    pub async fn fetch(url: &str) -> Result<Self, Error> {
        let client = Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::XmlParse(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::XmlParse(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        let xml_text = response
            .text()
            .await
            .map_err(|e| Error::XmlParse(e.to_string()))?;

        Self::parse(&xml_text)
    }

    /// Parses an XML string into a `Catalog`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::catalog::Catalog;
    ///
    /// let xml_data = r#"<?xml version="1.0" encoding="UTF-8"?>
    /// <catalog version="1.0">
    ///     <file name="model-a.gguf" file_size="100" chunk_size="50">
    ///         <mirrors>
    ///             <mirror src="https://mirror1.com/a.gguf"/>
    ///         </mirrors>
    ///         <hash>hash</hash>
    ///     </file>
    ///     <!-- Duplicate entry ignored -->
    ///     <file name="model-a.gguf" file_size="100" chunk_size="50">
    ///         <hash>hash</hash>
    ///     </file>
    /// </catalog>"#;
    ///
    /// let catalog = Catalog::parse(xml_data).unwrap();
    /// assert_eq!(catalog.files().len(), 1);
    ///
    /// let file = catalog.get("model-a.gguf");
    /// assert!(file.is_some());
    /// assert_eq!(
    ///     file.unwrap().mirror_urls(),
    ///     vec!["https://mirror1.com/a.gguf".to_string()]
    /// );
    /// ```
    pub fn parse(xml_str: &str) -> Result<Self, Error> {
        let raw_catalog: Catalog = quick_xml::de::from_str(xml_str)
            .map_err(|e| Error::XmlParse(e.to_string()))?;

        let mut seen_names = HashSet::new();
        let mut unique_files = Vec::new();

        for file in raw_catalog.files {
            if seen_names.contains(&file.name) {
                eprintln!(
                    "Warning: Duplicate file entry '{}' skipped.",
                    file.name
                );
                continue;
            }
            seen_names.insert(file.name.clone());
            unique_files.push(file);
        }

        Ok(Catalog {
            version: raw_catalog.version,
            files: unique_files,
        })
    }

    /// Returns a reference slice of all file entries in the catalog.
    pub fn files(&self) -> &[File] {
        &self.files
    }

    /// Finds a file entry within the catalog by name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::catalog::Catalog;
    /// use laded::file::File;
    ///
    /// let catalog = Catalog {
    ///     version: "1.0".to_string(),
    ///     files: vec![File {
    ///         name: "test.bin".to_string(),
    ///         file_size: 10,
    ///         chunk_size: 10,
    ///         title: None,
    ///         family: None,
    ///         model_size: None,
    ///         quantization: None,
    ///         adaptation: None,
    ///         parameters: None,
    ///         description: None,
    ///         mirrors: None,
    ///         hash: String::new(),
    ///     }],
    /// };
    ///
    /// assert!(catalog.get("test.bin").is_some());
    /// assert!(catalog.get("missing.bin").is_none());
    /// ```
    pub fn get(&self, name: &str) -> Option<&File> {
        self.files.iter().find(|f| f.name == name)
    }

    /// Deprecated backward compatibility alias for `get`.
    #[deprecated(since = "1.1.0", note = "Use `get` instead")]
    pub fn find_file(&self, name: &str) -> Option<&File> {
        self.get(name)
    }
}
