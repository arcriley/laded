/*  src/directory.rs  Recursive directory structure within package catalogs.
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

use crate::file::File;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a directory node inside a package, which can contain files
/// and recursively nested subdirectories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "file", default)]
    pub files: Vec<File>,

    #[serde(rename = "directory", default)]
    pub directories: Vec<Directory>,
}

impl Directory {
    /// Collects all files contained in this directory and its subdirectories.
    /// File names are updated with their relative paths (e.g., `weights/model-00001.safetensors`).
    pub fn all_files(&self) -> Vec<File> {
        let mut result = Vec::new();

        // Add immediate child files with path prefix
        for file in &self.files {
            let mut file_clone = file.clone();
            file_clone.name = PathBuf::from(&self.name)
                .join(&file.name)
                .to_string_lossy()
                .to_string();
            result.push(file_clone);
        }

        // Recursively traverse subdirectories
        for dir in &self.directories {
            for mut file in dir.all_files() {
                file.name = PathBuf::from(&self.name)
                    .join(&file.name)
                    .to_string_lossy()
                    .to_string();
                result.push(file);
            }
        }

        result
    }
}
