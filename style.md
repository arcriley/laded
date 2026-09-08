# STYLE.md — Development & Style Guidelines

This document outlines the coding standards, documentation practices, and testing requirements for this project. These guidelines ensure consistency across human contributions and automated code generation models.

---

## File Headers & Licensing

Every source file must begin with a standardized MIT copyright header specifying the module name, description, year, and copyright holder.

/*  src/example.rs  Brief description of module functionality.
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

---

## Line Length & Column Limits

* **Maximum Line Width:** All code, comments, and documentation lines should rarely exceed 79 characters.
* **PEP 8 Compliance:** Follow PEP 8 guidelines for formatting and line breaks (with a hard cap at 80 characters rather than 79).
* **Wrapping Strategy:** When breaking long statements or function signatures, indent continuation lines aligned vertically or using hanging indents.

---

## Integration Testing Standards (`tests/`)

* **Crate Imports:** Integration tests under the `tests/` directory run as external consumers of the library. Never use `use crate::...` inside `tests/`. Always reference items using the library crate name (e.g., `use laded::error::Error;`).
* **Test Documentation Header:** Every test function must contain a structured doc-comment detailing its purpose and execution method:

/// Short summary of test purpose.
///
/// **What it tests:**
/// Description of the exact invariant, branch, or error state being verified.
///
/// **How it tests:**
/// Step-by-step logic detailing mock setups, inputs provided, and assertions made.
#[test]
fn test_feature_behavior() {
    // ...
}

---

## Inline Documentation & Doc Tests

* All public functions, structs, enums, and trait implementations must be documented using triple-slash (`///`) doc comments.
* Include runnable `# Examples` blocks in doc comments for public API methods to ensure docs double as executable tests (`cargo test --doc`).

/// Creates a new instance of `Example`.
///
/// # Examples
///
/// ```rust
/// use laded::example::Example;
///
/// let instance = Example::new();
/// assert!(instance.is_valid());
/// ```
pub fn new() -> Self {
    // ...
}

---

## Error Handling & Branch Coverage

* **Eliminate Dead Code:** Avoid fallback branches or conditions checked against invariants guaranteed by prior code (e.g., checking `if !file_name.is_empty()` after an `ok_or_else` check that guarantees `file_name` is non-empty).
* **Option & Collection Unwrapping:** Derive struct fields or optional elements directly from collection properties (e.g., checking `if items.is_empty()` rather than checking metadata strings).
* **Error Enums:** Explicitly map external errors (such as `std::io::Error` or `quick_xml::Error`) to custom project error types via `crate::error::Error`.

---

## Pre-Release Verification Checklist

Prior to publishing or merging PRs, ensure the following pass locally:

* `cargo test` — All unit and integration tests pass.
* `cargo test --doc` — All documentation code examples execute successfully.
* `cargo clippy -- -D warnings` — Code passes static analysis without warnings.
* `cargo llvm-cov` — Coverage meets target thresholds without unused branches or functions.
