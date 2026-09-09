/*  utility/laded.rs  CLI utility for laded
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

use clap::{Parser, Subcommand};
use laded::builder::Builder;
use laded::catalog::Catalog;
use laded::downloader::Downloader;
use laded::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const DEFAULT_CHUNK_SIZE: u32 = 1048576; // 1MB default chunk size

#[derive(Parser)]
#[command(name = "laded")]
#[command(about = "CLI tool to inspect, pack, and fetch lading files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack a file into a .lading XML catalog
    Pack {
        /// Input file path to generate catalog for
        #[arg(long)]
        input: PathBuf,

        /// Output .lading catalog path (writes to stdout if omitted)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Title metadata field
        #[arg(long)]
        title: Option<String>,

        /// Family metadata field
        #[arg(long)]
        family: Option<String>,

        /// Quantization metadata field
        #[arg(long)]
        quantization: Option<String>,

        /// Mirror URL endpoints
        #[arg(long = "mirror")]
        mirrors: Vec<String>,
    },
    /// List available files in a local or remote catalog
    List {
        /// Path or URL to the .lading catalog file
        #[arg(long)]
        catalog: String,
    },
    /// Inspect details for a specific file entry in a catalog
    Info {
        /// Path or URL to the .lading catalog file
        #[arg(long)]
        catalog: String,

        /// File entry name
        #[arg(long)]
        name: String,
    },
    /// Download a file entry from a catalog
    Get {
        /// Path or URL to the .lading catalog file
        #[arg(long)]
        source: String,

        /// Specific file name to fetch from catalog
        #[arg(long)]
        name: Option<String>,

        /// Destination output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            input,
            output,
            title,
            family,
            quantization,
            mirrors,
        } => {
            let mut builder = Builder::new();
            builder.add_file_from_disk(
                &input,
                DEFAULT_CHUNK_SIZE,
                title,
                family,
                None, // model_size
                quantization,
                None, // adaptation
                None, // parameters
                None, // description
                mirrors,
            )?;

            let xml_out = builder.build()?;

            if let Some(out_path) = output {
                fs::write(out_path, xml_out)?;
            } else {
                io::stdout().write_all(xml_out.as_bytes())?;
            }
        }
        Commands::List { catalog } => {
            let cat = load_catalog(&catalog).await?;
            println!("Catalog Version: {}", cat.version);
            for file in cat.files() {
                println!("File: {}", file.name);
                if let Some(ref f) = file.family {
                    println!("Family: {}", f);
                }
                if let Some(ref q) = file.quantization {
                    println!("Quantization: {}", q);
                }
                for url in file.mirror_urls() {
                    println!("  - {}", url);
                }
            }
        }
        Commands::Info { catalog, name } => {
            let cat = load_catalog(&catalog).await?;
            if let Some(file) = cat.get(&name) {
                println!("Name:         {}", file.name);
                println!("File Size:    {} bytes", file.file_size);
                println!("Chunk Size:   {} bytes", file.chunk_size);
                if let Some(ref title) = file.title {
                    println!("Title:        {}", title);
                }
                if let Some(ref quant) = file.quantization {
                    println!("Quantization: {}", quant);
                }
                println!("Mirrors:");
                for url in file.mirror_urls() {
                    println!("  - {}", url);
                }
            } else {
                eprintln!("File '{}' not found in catalog.", name);
                std::process::exit(1);
            }
        }
        Commands::Get {
            source,
            name,
            output,
        } => {
            let cat = load_catalog(&source).await?;
            let file_entry = if let Some(target_name) = name {
                cat.get(&target_name).ok_or_else(|| {
                    format!("File '{}' not found in catalog", target_name)
                })?
            } else {
                let msg = "Catalog contains no file entries";
                cat.files()
                    .first()
                    .ok_or_else(|| msg.to_string())?
            };

            let dst_path = output.unwrap_or_else(|| {
                PathBuf::from(&file_entry.name)
            });

            let downloader = Downloader::new();
            downloader
                .download_file(
                    file_entry,
                    None,
                    &dst_path,
                    |_read, _total| {},
                )
                .await?;

            eprintln!("Download completed successfully.");
        }
    }

    Ok(())
}

/// Helper function to load a Catalog
/// This supports either a remote URL or local file path
async fn load_catalog(source: &str) -> Result<Catalog, Error> {
    if source.starts_with("http://") || source.starts_with("https://") {
        Catalog::fetch(source).await
    } else {
        let content = fs::read_to_string(PathBuf::from(source))
            .map_err(|e| Error::XmlParse(e.to_string()))?;
        Catalog::parse(&content)
    }
}
