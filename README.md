# rcue

[![Build Status](https://travis-ci.org/gyng/rcue.svg?branch=master)](https://travis-ci.org/gyng/rcue)

Simple CUE sheet reader for Rust. Compiles on stable.

This library reads some CUE files fine, but is missing one important feature. Right now, indentation is treated as insignificant.

For example, if `REM` fields appear after a `TRACK` field (but are indented to the `FILE`'s level, it will be wrongly assigned to the `TRACK` instead.

See generated documentation (`cargo doc`) or tests in [`parser.rs`](src/parser.rs) for usage and some examples.

## Usage

Add this to your `[dependencies]`

```
rcue = { git = "https://github.com/gyng/rcue" }
```

And in your `lib.rs` or `main.rs`:

```rust
extern crate rcue;

use rcue::parser::parse;
use rcue::parser::parse_from_file;

fn main() {
    let cue = parse_from_file("test/fixtures/good.cue", true).unwrap();
    assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
}
```

## TODO

* Stricter strict mode
* (Significant indentation) support
* Serializer
* time::Duration for timestamps
* More complete documentation
