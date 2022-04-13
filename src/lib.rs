#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

//! rcue is a simple CUE sheet reader.
//!
//! This library reads some CUE files fine, but is missing one important feature.
//!
//! Right now, indentation is treated as insignificant (= no proper contextual support).
//! This means if `REM` fields appear after a `TRACK` field (but are indented to the `FILE`'s level,
//! it will be wrongly assigned to the `TRACK` instead.
//!
//! ## Usage
//!
//! ```rust
//! extern crate rcue;
//!
//! use rcue::parser::parse_from_file;
//! use rcue::parser::parse;
//!
//! fn main() {
//!     let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
//!     assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
//!
//!     let file = std::fs::File::open("test/fixtures/unicode.cue").unwrap();
//!     let mut buf_reader = std::io::BufReader::new(file);
//!     let cue = parse(&mut buf_reader, true).unwrap();
//!     assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
//! }
//! ```
//!
//! See [`rcue::parser::parse`](parser/fn.parse.html) or
//! [`rcue::parser::parse_from_file`](parser/fn.parse_from_file.html) for usage.
//!
//! [GitHub repository](https://github.com/gyng/rcue)

/// Structs and types
pub mod cue;
/// Errors module
pub mod errors;
/// Parser implementation
pub mod parser;
/// Utility functions
pub mod util;
