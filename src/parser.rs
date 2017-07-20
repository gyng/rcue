use itertools::Itertools;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Rem(String, String),
    Performer(String),
    Title(String),
    File(String, String),
    Track(String, String),
    Index(String, String),
    Unknown(String),
    None,
}

#[derive(Clone, Debug)]
struct Track {
    no: String,
    format: String,
    title: Option<String>,
    performer: Option<String>,
    indices: Vec<(String, String)>,
}

impl Track {
    pub fn new(no: &str, format: &str) -> Self {
        Self {
            no: no.to_string(),
            format: format.to_string(),
            title: None,
            performer: None,
            indices: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct CueFile {
    format: String,
    tracks: Vec<Track>,
}

impl CueFile {
    pub fn new(format: &str) -> Self {
        Self {
            tracks: Vec::new(),
            format: format.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Cue {
    files: Vec<CueFile>,
    title: Option<String>,
    performer: Option<String>,
    comments: Vec<(String, String)>, // are REM fields unique?
}

impl Cue {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            title: None,
            performer: None,
            comments: Vec::new(),
        }
    }
}

// fn parse_from_file() -> Result<(), String> {
//     let file = File::open("test.cue").map_err(|e| e.to_string())?;
//     let mut buf_reader = BufReader::new(file);
// }

fn parse() -> Result<(), String> {
    let file = File::open("test.cue").map_err(|e| e.to_string())?;
    let mut buf_reader = BufReader::new(file);

    let mut cue = Cue::new();

    fn last_file(cue: &mut Cue) -> Option<&mut CueFile> {
        cue.files.last_mut()
    }

    fn last_track(cue: &mut Cue) -> Option<&mut Track> {
        last_file(cue).and_then(|f| f.tracks.last_mut())
    }

    for line in buf_reader.lines() {
        if let Ok(l) = line {
            let token = tokenize_line(&l);
            println!("{0:<60} {1:?}", l, token);

            match token {
                Token::Rem(field, value) => cue.comments.push((field, value)),
                Token::File(file, format) => {
                    cue.files.push(CueFile::new(&format));
                }
                Token::Track(idx, mode) => {
                    if let Some(file) = last_file(&mut cue) {
                        file.tracks.push(Track::new(&idx, &mode));
                    }
                }
                Token::Title(title) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().title = Some(title);
                    } else {
                        cue.title = Some(title)
                    }
                }
                Token::Performer(performer) => {
                    // this double check might be able to go away under non-lexical lifetimes
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().performer = Some(performer);
                    } else {
                        cue.performer = Some(performer);
                    }
                }
                Token::Index(idx, time) => {
                    if let Some(track) = last_track(&mut cue) {
                        track.indices.push((idx, time));
                    }
                }
                _ => println!("Did not parse line: {:?}", l),
            }
        }
    }

    println!("\ncue:\n{:#?}", cue);

    Ok(())
}

fn tokenize_line(line: &str) -> Token {
    let mut tokens = line.trim().split_whitespace();

    match tokens.next() {
        Some("REM") => {
            let key = tokens.next().unwrap().to_string();
            let val = tokens.join(" ");
            Token::Rem(key, val)
        }
        Some("TITLE") => {
            let val = tokens.join(" ");
            Token::Title(val)
        }
        Some("FILE") => {
            let l: Vec<_> = tokens.collect();
            let (&format, vals) = l.split_last().unwrap();
            let val = vals.join(" ");
            Token::File(val, format.to_string())
        }
        Some("PERFORMER") => {
            let val = tokens.join(" ");
            Token::Performer(val)
        }
        Some("TRACK") => {
            let val = tokens.next().unwrap().to_string();
            let mode = tokens.next().unwrap().to_string();
            Token::Track(val, mode)
        }
        Some("INDEX") => {
            let val = tokens.next().unwrap().to_string();
            let time = tokens.next().unwrap().to_string();
            Token::Index(val, time)
        }
        Some(other) => Token::Unknown(line.to_string()),
        _ => Token::None,
    }
}

#[test]
fn check_string_between_quotes() {
    parse();
    assert!(false);
}