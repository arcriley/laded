/*  src/package.rs  Package container node inside XML catalogs.
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

use crate::directory::Directory;
use crate::file::File;
use serde::{Deserialize, Serialize};

/// Represents a package entry inside a catalog, holding top-level metadata,
/// root files, and subdirectories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    #[serde(rename = "@title")]
    pub title: String,

    #[serde(rename = "@description")]
    pub description: Option<String>,

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

    #[serde(rename = "file", default)]
    pub files: Vec<File>,

    #[serde(rename = "directory", default)]
    pub directories: Vec<Directory>,
}

impl Package {
    /// Returns a flat vector of all files contained within this package,
    /// flattening nested directories and resolving their relative file paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use laded::package::Package;
    /// use laded::directory::Directory;
    /// use laded::file::File;
    ///
    /// let package = Package {
    ///     title: "Test Package".to_string(),
    ///     description: None,
    ///     family: None,
    ///     model_size: None,
    ///     quantization: None,
    ///     adaptation: None,
    ///     parameters: None,
    ///     files: vec![File {
    ///         name: "root.json".to_string(),
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
    ///     directories: vec![],
    /// };
    ///
    /// assert_eq!(package.all_files().len(), 1);
    /// assert_eq!(package.all_files()[0].name, "root.json");
    /// ```
    pub fn all_files(&self) -> Vec<File> {
        let mut all = self.files.clone();
        for dir in &self.directories {
            all.extend(dir.all_files());
        }
        all
    }

    /// Computes total payload size in bytes for all files in this package.
    pub fn total_size(&self) -> u64 {
        self.all_files().iter().map(|f| f.file_size).sum()
    }
}
