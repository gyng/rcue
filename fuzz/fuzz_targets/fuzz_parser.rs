#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rcue;

use std::io::BufReader;

fuzz_target!(|data: &[u8]| {
    let mut reader = BufReader::new(data);
    let _ = rcue::parser::parse(&mut reader, false);
});
