use itertools::Itertools;
// use regex::Regex;

use std::fs::File;
use std::io::{BufRead, BufReader};

use errors::CueError;
use util::unescape_string;

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
pub struct Track {
    no: String,
    format: String,
    title: Option<String>,
    performer: Option<String>,
    indices: Vec<(String, String)>,
    unknown: Vec<String>,
}

impl Track {
    pub fn new(no: &str, format: &str) -> Self {
        Self {
            no: no.to_string(),
            format: format.to_string(),
            title: None,
            performer: None,
            indices: Vec::new(),
            unknown: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CueFile {
    file: String,
    format: String,
    tracks: Vec<Track>,
}

impl CueFile {
    pub fn new(file: &str, format: &str) -> Self {
        Self {
            file: file.to_string(),
            tracks: Vec::new(),
            format: format.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Cue {
    files: Vec<CueFile>,
    title: Option<String>,
    performer: Option<String>,
    comments: Vec<(String, String)>, // are REM fields unique?
    unknown: Vec<String>,
}

impl Cue {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            title: None,
            performer: None,
            comments: Vec::new(),
            unknown: Vec::new(),
        }
    }
}

#[allow(dead_code)]
pub fn parse_from_file(path: &str, strict: bool) -> Result<Cue, CueError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    parse(Box::new(buf_reader), strict)
}

#[allow(dead_code)]
pub fn parse(buf_reader: Box<BufRead>, strict: bool) -> Result<Cue, CueError> {
    let mut cue = Cue::new();

    fn last_file(cue: &mut Cue) -> Option<&mut CueFile> {
        cue.files.last_mut()
    }

    fn last_track(cue: &mut Cue) -> Option<&mut Track> {
        last_file(cue).and_then(|f| f.tracks.last_mut())
    }

    for (i, line) in buf_reader.lines().enumerate() {
        if let Ok(l) = line {
            let token = tokenize_line(&l);

            match token {
                Ok(Token::Rem(field, value)) => cue.comments.push((field, value)),
                Ok(Token::File(file, format)) => {
                    cue.files.push(CueFile::new(&file, &format));
                }
                Ok(Token::Track(idx, mode)) => {
                    if let Some(file) = last_file(&mut cue) {
                        file.tracks.push(Track::new(&idx, &mode));
                    }
                }
                Ok(Token::Title(title)) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().title = Some(title);
                    } else {
                        cue.title = Some(title)
                    }
                }
                Ok(Token::Performer(performer)) => {
                    // this double check might be able to go away under non-lexical lifetimes
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().performer = Some(performer);
                    } else {
                        cue.performer = Some(performer);
                    }
                }
                Ok(Token::Index(idx, time)) => {
                    if let Some(track) = last_track(&mut cue) {
                        track.indices.push((idx, time));
                    }
                }
                Ok(Token::Unknown(line)) => {
                    if strict {
                        println!(
                            "Strict mode failure: Unknown token - did not parse line {}: {:?}",
                            i + 1,
                            l
                        );
                        return Err(CueError::Parse("strict mode failure: bad line".to_string()));
                    }

                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().unknown.push(line);
                    } else {
                        cue.unknown.push(line)
                    }
                }
                _ => {
                    if strict {
                        println!(
                            "Strict mode failure: Bad line - did not parse line {}: {:?}",
                            i + 1,
                            l
                        );
                        return Err(CueError::Parse("strict mode failure: bad line".to_string()));
                    }
                    println!("Bad line - did not parse line {}: {:?}", i + 1, l);
                }
            }
        }
    }

    Ok(cue)
}

#[allow(dead_code)]
fn tokenize_line(line: &str) -> Result<Token, CueError> {
    // Do not use split_whitespace to avoid string mutation as tokens are joined back using normal spaces
    let mut tokens = line.trim().split(" ");

    macro_rules! next_token {
        ($tokens:ident, $error:expr) => (
            tokens.next().ok_or(CueError::Parse($error.to_string()))?.to_string()
        )
    }

    match tokens.next() {
        Some("REM") => {
            let key = next_token!(tokens, "missing REM key");
            let val = unescape_string(&tokens.join(" "));
            Ok(Token::Rem(key, val))
        }
        Some("TITLE") => {
            let val = unescape_string(&tokens.join(" "));
            Ok(Token::Title(val))
        }
        Some("FILE") => {
            let l: Vec<_> = tokens.collect();
            let (&format, vals) = l.split_last().unwrap();
            let val = unescape_string(&vals.join(" "));
            Ok(Token::File(val, format.to_string()))
        }
        Some("PERFORMER") => {
            let val = unescape_string(&tokens.join(" "));
            Ok(Token::Performer(val))
        }
        Some("TRACK") => {
            let val = next_token!(tokens, "missing TRACK number");
            let mode = next_token!(tokens, "missing TRACK mode");
            Ok(Token::Track(val, mode))
        }
        Some("INDEX") => {
            let val = next_token!(tokens, "missing INDEX number");
            let time = next_token!(tokens, "missing INDEX time");
            Ok(Token::Index(val, time))
        }
        Some(_) => Ok(Token::Unknown(line.to_string())),
        _ => Ok(Token::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_string_between_quotes() {
        let cue = parse_from_file("test.cue", true);
        println!("{:#?}", cue);
        assert!(false);
    }
}
