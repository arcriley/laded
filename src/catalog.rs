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
    /// assert_eq!(catalog.files.len(), 1);
    ///
    /// let file = catalog.find_file("model-a.gguf");
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
    /// assert!(catalog.find_file("test.bin").is_some());
    /// assert!(catalog.find_file("missing.bin").is_none());
    /// ```
    pub fn find_file(&self, name: &str) -> Option<&File> {
        self.files.iter().find(|f| f.name == name)
    }
}
