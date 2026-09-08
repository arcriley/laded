/*  src/utility/laded.rs  CLI tool for catalog creation and downloading.
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


use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use laded::{Builder, Catalog, Downloader};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "laded")]
#[command(
    about = "Chunked file downloader and XML catalog generator",
    long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack a local file into an XML catalog manifest
    Pack {
        /// Local file path to index into catalog
        #[arg(short, long)]
        input: PathBuf,

        /// Output catalog XML file path (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Chunk size in bytes (default: 10485760 / 10MB)
        #[arg(short, long, default_value_t = 10485760)]
        chunk_size: u32,

        /// Optional descriptive title
        #[arg(long)]
        title: Option<String>,

        /// Model family (e.g. "Llama")
        #[arg(long)]
        family: Option<String>,

        /// Parameter size indicator (e.g. "7B")
        #[arg(long)]
        model_size: Option<String>,

        /// Quantization format (e.g. "Q5_K_M")
        #[arg(long)]
        quantization: Option<String>,

        /// Fine-tuning/adaptation descriptor
        #[arg(long)]
        adaptation: Option<String>,

        /// Additional key/value model parameters
        #[arg(long)]
        parameters: Option<String>,

        /// Verbose model or file description
        #[arg(long)]
        description: Option<String>,

        /// Mirror download URLs
        #[arg(short, long)]
        mirror: Vec<String>,
    },

    /// Inspect entries in an existing catalog file
    List {
        /// Target catalog XML file path
        #[arg(short, long)]
        catalog: PathBuf,
    },

    /// Download a file entry defined in a catalog
    Get {
        /// Target catalog XML file path
        #[arg(short, long)]
        source: PathBuf,

        /// Name of file entry in catalog to download
        #[arg(short, long)]
        name: Option<String>,

        /// Runtime mirror URL override
        #[arg(short, long)]
        url: Option<String>,

        /// Local output destination path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
            input,
            output,
            chunk_size,
            title,
            family,
            model_size,
            quantization,
            adaptation,
            parameters,
            description,
            mirror,
        } => {
            let mut builder = Builder::new();
            builder
                .add_file_from_disk(
                    &input,
                    chunk_size,
                    title,
                    family,
                    model_size,
                    quantization,
                    adaptation,
                    parameters,
                    description,
                    mirror,
                )
                .context("Failed to index input file into catalog")?;

            let catalog_xml =
                builder.build().context("Failed to build catalog XML")?;

            if let Some(out_path) = output {
                fs::write(&out_path, catalog_xml)
                    .context("Failed to write output file")?;
                eprintln!("Catalog successfully written to {:?}", out_path);
            } else {
                io::stdout().write_all(catalog_xml.as_bytes())?;
            }
        }

        Commands::List { catalog } => {
            let xml_content = fs::read_to_string(&catalog)
                .context("Failed to read catalog file")?;
            let parsed = Catalog::parse(&xml_content)
                .context("Failed to parse catalog XML")?;

            println!("Catalog Version: {}", parsed.version);
            println!("\nFiles ({}):", parsed.files.len());
            for file in &parsed.files {
                println!("- Name: {}", file.name);
                println!("  Size: {} bytes", file.file_size);
                println!("  Chunk Size: {} bytes", file.chunk_size);
                if let Some(f) = &file.family {
                    println!("  Family: {}", f);
                }
                if let Some(q) = &file.quantization {
                    println!("  Quantization: {}", q);
                }
                let mirrors = file.mirror_urls();
                if !mirrors.is_empty() {
                    println!("  Mirrors ({}):", mirrors.len());
                    for m in mirrors {
                        println!("    * {}", m);
                    }
                }
            }
        }

        Commands::Get {
            source,
            name,
            url,
            output,
        } => {
            let content = fs::read_to_string(&source)
                .context("Failed to read catalog file")?;
            let catalog = Catalog::parse(&content)
                .context("Failed to parse catalog XML")?;

            let target_file = if let Some(target_name) = name {
                catalog.find_file(&target_name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "File '{}' not found in catalog", 
                        target_name)
                })?
            } else if catalog.files.len() == 1 {
                &catalog.files[0]
            } else {
                anyhow::bail!(
                    "Multiple files in catalog. Specify target using --name"
                );
            };

            let dest_path =
                output.unwrap_or_else(|| PathBuf::from(&target_file.name));

            eprintln!("Downloading: {}", target_file.name);
            eprintln!("Destination: {:?}", dest_path);

            let downloader = Downloader::new();
            downloader
                .download_file(
                    target_file,
                    url.as_deref(),
                    &dest_path,
                    |downloaded, total| {
                        let percent = if total > 0 {
                            (downloaded as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };
                        eprint!(
                            "\rProgress: {} / {} bytes ({:.2}%)",
                            downloaded, total, percent
                        );
                        let _ = io::stderr().flush();
                    },
                )
                .await
                .context("Download failed")?;

            eprintln!("\nDownload completed successfully.");
        }
    }

    Ok(())
}
