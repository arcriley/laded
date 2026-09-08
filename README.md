# laded

`laded` is an asynchronous chunked file retriever and catalog archiving engine
written in Rust.

While designed to handle generic binary artifacts, `laded` specifically
addresses the reliability and bandwidth constraints of distributing massive
Large Language Model (LLM) weights (e.g., GGUF, SafeTensors). 

Transferring multi-gigabyte files over standard HTTP connections presents two
major failure modes: single-byte corruption invalidating an entire download,
and mirror bandwidth throttling. `laded` solves both by decomposing files into
catalog-defined byte ranges, verifying SHA-256 digests on a per-chunk basis,
and streaming chunks in parallel across multiple redundant mirrors.

---

## Key Technical Features

* **Granular SHA-256 Integrity Verification:** Every file chunk is validated
against a pre-computed Base64-encoded SHA-256 digest immediately upon
retrieval. Corrupted chunks trigger localized retries without discarding
previously downloaded byte ranges.
* **Resilient Multi-Mirror Acceleration:** Retrieves chunks across configured
mirror endpoints to maximize local download bandwidth and mitigate single-host
rate limiting.
* **Deterministic XML Catalog Schema:** Parses structured `<catalog>` manifests
detailing metadata parameters (quantization, parameter scale, adaptation)
alongside file chunk maps.
* **CLI & Native Library Crate:** Exposes both a standalone command-line binary
(`laded`) and a native Rust API (`laded::Catalog`, `laded::Builder`,
`laded::Downloader`) for direct integration into downstream applications like
`gneural`.

---

## Catalog Architecture

Catalogs are structured XML manifests declaring target file attributes, chunk
sizes, mirror endpoints, and whitespace-delimited Base64 SHA-256 digests:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<catalog version="1.0">
  <file 
    name="llama-3-8b-instruct.Q5_K_M.gguf" 
    file_size="5730000000" 
    chunk_size="10485760"
    title="Llama 3 8B Instruct"
    family="Llama"
    model_size="8B"
    quantization="Q5_K_M"
    description="Fine-tuned instruct model quantized for edge deployment.">
    <mirrors>
      <mirror src="[https://mirror1.example.com/models/llama-3-8b.gguf](https://mirror1.example.com/models/llama-3-8b.gguf)"/>
      <mirror src="[https://mirror2.example.com/models/llama-3-8b.gguf](https://mirror2.example.com/models/llama-3-8b.gguf)"/>
    </mirrors>
    <hash>
      uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek=
      41920b348e0c6ff2ef9b7e3ee9308726aa5250fa717883e073ff6a936a9325a4=
    </hash>
  </file>
</catalog>
```

### Build and Install

#### Prerequisites

* Rust edition 2021 (1.70+ recommended)
* Cargo package manager
* Meson build system (>= 1.0.0) & Ninja (optional, for Meson workflow)

#### Cargo Workflow (Standard Rust Toolchain)

Build and install the binary executable directly to `~/.cargo/bin`:

```bash
cargo install --path .
```

To build standard debug or release binaries locally without installing:

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release
```

The output binaries will be generated at `./target/debug/laded` and
`./target/release/laded` respectively.

#### Meson Build System (System & Distro Packaging)

Configure the build directory and install using Meson and Ninja:

```bash
# Configure build directory
meson setup build --prefix=/usr/local

# Compile targets
ninja -C build

# Install binary to /usr/local/bin
sudo ninja -C build install
```

---

### Testing & Documentation

#### Running Tests

Execute the full unit and integration test suite:

```bash
cargo test
```

To run exclusively the API documentation tests (doctests):

```bash
cargo test --doc
```

To execute the test suite through the Meson test runner:

```bash
meson test -C build
```

#### Generating Documentation

Build the HTML documentation locally, including all internal and private API
items, and automatically open it in your default browser:

```bash
cargo doc --no-deps --open
```

To build production-ready documentation with embedded cross-references:

```bash
cargo doc --no-deps --release
```

The generated HTML documentation will be located at `./target/doc/laded/index.html`.
