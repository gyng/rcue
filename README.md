# rcue

[![Build Status](https://travis-ci.org/gyng/rcue.svg?branch=master)](https://travis-ci.org/gyng/rcue)

[Documentation](https://gyng.github.io/rcue)

Simple CUE sheet reader for Rust. Compiles on stable.

This library reads some CUE files fine, but is missing one important feature. Right now, indentation is treated as insignificant (= no proper contextual support).

For example, if `REM` fields appear after a `TRACK` field (but are indented to the `FILE`'s level, it will be wrongly assigned to the `TRACK` instead.

## Usage

See [generated documentation](https://gyng.github.io/rcue) or tests in [`parser.rs`](src/parser.rs) for usage and some examples.

Add this to your `Cargo.toml` under `[dependencies]`

```
rcue = { git = "https://github.com/gyng/rcue" }
```

In your program:

```rust
extern crate rcue;

use rcue::parser::parse;
use rcue::parser::parse_from_file;

fn main() {
    let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
    assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));

    let file = std::fs::File::open("test/fixtures/unicode.cue").unwrap();
    let mut buf_reader = std::io::BufReader::new(file);
    let cue = parse(&mut buf_reader, true).unwrap();
    assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
}
```

## Verbose log information
For verbose logging to STDOUT and details on skipped lines in lenient mode, run rcue with the environment variable `RCUE_LOG` set to `1`. For example:

```
# myapp.rs
RCUE_LOG=1 cargo run
```

## Fuzzing

The parser fuzz test for rcue can be run using `cargo fuzz` in nightly.

```
cargo install cargo-fuzz -f
cargo +nightly fuzz run fuzz_parser
```

## Clippy

Run clippy using

```
rustup install nightly # if not installed
rustup update nightly
cargo +nightly install clippy --force # --force to update
rustup run nightly cargo clippy
```

## TODO

* Significant indentation/context support
* Serializer
* Clean up parsing
