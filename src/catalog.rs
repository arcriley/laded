/*  src/catalog.rs  XML catalog parser and package entry container.
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
use crate::package::Package;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "catalog")]
pub struct Catalog {
    #[serde(rename = "@version")]
    pub version: String,

    #[serde(rename = "package", default)]
    pub packages: Vec<Package>,
}

impl Catalog {
    /// Asynchronously fetches an XML catalog file from a remote URL.
    ///
    /// # Arguments
    /// - `url`: Public HTTP/HTTPS endpoint pointing to `.lading` file.
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
    ///     <package title="Llama 3 8B Instruct" description="Fine-tuned model">
    ///         <file name="llama-3.gguf" file_size="100" chunk_size="50">
    ///             <mirrors>
    ///                 <mirror src="[https://mirror1.com/a.gguf](https://mirror1.com/a.gguf)"/>
    ///             </mirrors>
    ///             <hash>hash</hash>
    ///         </file>
    ///     </package>
    /// </catalog>"#;
    ///
    /// let catalog = Catalog::parse(xml_data).unwrap();
    /// assert_eq!(catalog.packages().len(), 1);
    /// assert_eq!(catalog.packages()[0].title, "Llama 3 8B Instruct");
    /// ```
    pub fn parse(xml_str: &str) -> Result<Self, Error> {
        let raw_catalog: Catalog = quick_xml::de::from_str(xml_str)
            .map_err(|e| Error::XmlParse(e.to_string()))?;

        Ok(raw_catalog)
    }

    /// Returns a reference slice of all package entries in the catalog.
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }

    /// Finds a package entry within the catalog by title.
    pub fn get_package(&self, title: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.title == title)
    }
}
