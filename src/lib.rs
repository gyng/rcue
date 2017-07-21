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
//! Add this to your `Cargo.toml` under `[dependencies]`
//!
//! ```text
//! rcue = { git = "https://github.com/gyng/rcue" }
//! ```
//!
//! Then in your program:
//!
//! ```rust
//! extern crate rcue;
//!
//! use rcue::parser::parse;
//! use rcue::parser::parse_from_file;
//!
//! fn main() {
//!     let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
//!     assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
//! }
//! ```
//!
//! See [`rcue::parser::parse`](parser/fn.parse.html) or
//! [`rcue::parser::parse_from_file`](parser/fn.parse_from_file.html) for usage.
//!
//! [GitHub repository](https://github.com/gyng/rcue)

extern crate itertools;
extern crate regex;
#[macro_use]
extern crate lazy_static;

pub mod errors;
pub mod parser;
pub mod util;
