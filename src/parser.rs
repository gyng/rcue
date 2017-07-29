use itertools::Itertools;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use errors::CueError;
use util::{timestamp_to_duration, unescape_string};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Rem(String, String),
    Catalog(String),
    Performer(String),
    Title(String),
    File(String, String),
    Track(String, String),
    Index(String, String),
    Pregap(String),
    Postgap(String),
    Unknown(String),
    None,
}

/// Represents a TRACK in a [`CueFile`](struct.CueFile.html).
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// track number
    pub no: String,
    /// track format (eg. AUDIO)
    pub format: String,
    pub title: Option<String>,
    pub performer: Option<String>,
    /// (index, timestamp)
    pub indices: Vec<(String, Duration)>,
    pub pregap: Option<Duration>,
    pub postgap: Option<Duration>,
    /// (key, value)
    pub comments: Vec<(String, String)>,
    /// raw lines from unhandled fields
    pub unknown: Vec<String>,
}

impl Track {
    /// Constructs a new [`Track`](struct.Track.html).
    pub fn new(no: &str, format: &str) -> Self {
        Self {
            no: no.to_string(),
            format: format.to_string(),
            title: None,
            performer: None,
            pregap: None,
            postgap: None,
            indices: Vec::new(),
            comments: Vec::new(),
            unknown: Vec::new(),
        }
    }
}

/// Represents a FILE in a [`Cue`](struct.Cue.html).
#[derive(Clone, Debug, PartialEq)]
pub struct CueFile {
    /// path to file
    pub file: String,
    /// format (eg. WAVE, MP3)
    pub format: String,
    pub tracks: Vec<Track>,
    /// (key, value)
    pub comments: Vec<(String, String)>,
}

impl CueFile {
    /// Constructs a new CueFile.
    pub fn new(file: &str, format: &str) -> Self {
        Self {
            file: file.to_string(),
            tracks: Vec::new(),
            format: format.to_string(),
            comments: Vec::new(),
        }
    }
}

/// Represents a CUE sheet.
#[derive(Clone, Debug, Default)]
pub struct Cue {
    pub files: Vec<CueFile>,
    pub title: Option<String>,
    pub performer: Option<String>,
    pub catalog: Option<String>,
    /// (key, value)
    pub comments: Vec<(String, String)>, // are REM fields unique?
    pub unknown: Vec<String>,
}

impl Cue {
    /// Constructs a new Cue.
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            title: None,
            performer: None,
            catalog: None,
            comments: Vec::new(),
            unknown: Vec::new(),
        }
    }
}

/// Parses a CUE file at `path` into a [`Cue`](struct.Cue.html) struct.
///
/// Strict mode (`strict: true`) will return a [`CueError`](../errors/enum.CueError.html) if invalid fields or extra lines are detected.
/// When not in strict mode, bad lines and fields will be skipped, and unknown
/// fields will be stored in [`Cue.unknown`](struct.Cue.html).
///
/// # Example
///
/// ```
/// use rcue::parser::parse_from_file;
///
/// let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
/// assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
/// ```
///
/// # Failures
///
/// Fails if the CUE file can not be parsed from the file.
#[allow(dead_code)]
pub fn parse_from_file(path: &str, strict: bool) -> Result<Cue, CueError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    parse(Box::new(buf_reader), strict)
}

/// Parses a [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html) into a [`Cue`](struct.Cue.html) struct.
///
/// Strict mode will return [`CueError`](../errors/enum.CueError.html) if invalid fields or extra lines are detected.
/// When not in strict mode, bad lines and fields will be skipped, and unknown
/// fields will be stored in [`Cue.unknown`](struct.Cue.html).
///
/// # Example
///
/// ```
/// use rcue::parser::parse;
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let file = File::open("test/fixtures/unicode.cue").unwrap();
/// let buf_reader = BufReader::new(file);
/// let cue = parse(Box::new(buf_reader), true).unwrap();
/// assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
/// ```
///
/// # Failures
///
/// Fails if the CUE file can not be parsed.
#[allow(dead_code)]
pub fn parse(buf_reader: Box<BufRead>, strict: bool) -> Result<Cue, CueError> {
    macro_rules! fail_if_strict {
        ($line_no:ident, $line:ident, $reason:expr) => (
            if strict {
                println!(
                    "Strict mode failure: did not parse line {}: {}\n\tReason: {:?}",
                    $line_no + 1,
                    $line,
                    $reason
                );
                return Err(CueError::Parse(format!("strict mode failure: {}", $reason)));
            }
        )
    }

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
                Ok(Token::Rem(field, value)) => {
                    let comment = (field, value);

                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().comments.push(comment);
                    } else if last_file(&mut cue).is_some() {
                        last_file(&mut cue).unwrap().comments.push(comment);
                    } else {
                        cue.comments.push(comment);
                    }
                }
                Ok(Token::File(file, format)) => {
                    cue.files.push(CueFile::new(&file, &format));
                }
                Ok(Token::Track(idx, mode)) => {
                    if let Some(file) = last_file(&mut cue) {
                        file.tracks.push(Track::new(&idx, &mode));
                    } else {
                        fail_if_strict!(i, l, "TRACK assigned to no FILE");
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
                        if let Ok(duration) = timestamp_to_duration(&time) {
                            track.indices.push((idx, duration));
                        } else {
                            fail_if_strict!(i, l, "bad INDEX timestamp");
                        }
                    } else {
                        fail_if_strict!(i, l, "INDEX assigned to no track");
                    }
                }
                Ok(Token::Pregap(time)) => {
                    if last_track(&mut cue).is_some() {
                        if let Ok(duration) = timestamp_to_duration(&time) {
                            last_track(&mut cue).unwrap().pregap = Some(duration);
                        } else {
                            fail_if_strict!(i, l, "bad PREGAP timestamp");
                        }
                    } else {
                        fail_if_strict!(i, l, "PREGAP assigned to no track");
                    }
                }
                Ok(Token::Postgap(time)) => {
                    if last_track(&mut cue).is_some() {
                        if let Ok(duration) = timestamp_to_duration(&time) {
                            last_track(&mut cue).unwrap().postgap = Some(duration);
                        } else {
                            fail_if_strict!(i, l, "bad PREGAP timestamp");
                        }
                    } else {
                        fail_if_strict!(i, l, "POSTGAP assigned to no track");
                    }
                }
                Ok(Token::Catalog(id)) => {
                    cue.catalog = Some(id);
                }
                Ok(Token::Unknown(line)) => {
                    fail_if_strict!(i, l, "unknown token");

                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().unknown.push(line);
                    } else {
                        cue.unknown.push(line)
                    }
                }
                _ => {
                    fail_if_strict!(i, l, "bad line");
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
    let mut tokens = line.trim().split(' ');

    macro_rules! next_token {
        ($tokens:ident, $error:expr) => (
            tokens.next().ok_or(CueError::Parse($error.to_string()))?.to_string()
        )
    }

    match tokens.next() {
        Some(t) => {
            let uppercase = t.to_uppercase();
            match uppercase.as_ref() {
                "REM" => {
                    let key = next_token!(tokens, "missing REM key");
                    let val = unescape_string(&tokens.join(" "));
                    Ok(Token::Rem(key, val))
                }
                "CATALOG" => {
                    let val = unescape_string(&tokens.join(" "));
                    Ok(Token::Catalog(val))
                }
                "TITLE" => {
                    let val = unescape_string(&tokens.join(" "));
                    Ok(Token::Title(val))
                }
                "FILE" => {
                    let l: Vec<_> = tokens.collect();
                    let (&format, vals) = l.split_last().ok_or_else(|| {
                        CueError::Parse("bare FILE token".to_string())
                    })?;
                    let val = unescape_string(&vals.join(" "));
                    Ok(Token::File(val, format.to_string()))
                }
                "PERFORMER" => {
                    let val = unescape_string(&tokens.join(" "));
                    Ok(Token::Performer(val))
                }
                "TRACK" => {
                    let val = next_token!(tokens, "missing TRACK number");
                    let mode = next_token!(tokens, "missing TRACK mode");
                    Ok(Token::Track(val, mode))
                }
                "PREGAP" => {
                    let val = next_token!(tokens, "missing PREGAP duration");
                    Ok(Token::Pregap(val))
                }
                "POSTGAP" => {
                    let val = next_token!(tokens, "missing POSTGAP duration");
                    Ok(Token::Postgap(val))
                }
                "INDEX" => {
                    let val = next_token!(tokens, "missing INDEX number");
                    let time = next_token!(tokens, "missing INDEX time");
                    Ok(Token::Index(val, time))
                }
                _ => {
                    if t.is_empty() {
                        Ok(Token::None)
                    } else {
                        Ok(Token::Unknown(line.to_string()))
                    }
                }
            }
        }
        _ => Ok(Token::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parsing_good_cue() {
        let cue = parse_from_file("test/fixtures/good.cue", true).unwrap();
        assert_eq!(cue.comments.len(), 4);
        assert_eq!(cue.comments[0], (
            "GENRE".to_string(),
            "Alternative".to_string(),
        ));
        assert_eq!(cue.comments[1], ("DATE".to_string(), "1991".to_string()));
        assert_eq!(cue.comments[2], (
            "DISCID".to_string(),
            "860B640B".to_string(),
        ));
        assert_eq!(cue.comments[3], (
            "COMMENT".to_string(),
            "ExactAudioCopy v0.95b4".to_string(),
        ));
        assert_eq!(cue.performer, Some("My Bloody Valentine".to_string()));
        assert_eq!(cue.title, Some("Loveless".to_string()));
        assert_eq!(cue.files.len(), 1);
        assert_eq!(cue.files[0].file, "My Bloody Valentine - Loveless.wav");
        assert_eq!(cue.files[0].format, "WAVE");
        assert_eq!(cue.files[0].tracks.len(), 2);
        assert_eq!(cue.files[0].tracks[0].no, "01".to_string());
        assert_eq!(cue.files[0].tracks[0].format, "AUDIO".to_string());
        assert_eq!(
            cue.files[0].tracks[0].title,
            Some("Only Shallow".to_string())
        );
        assert_eq!(
            cue.files[0].tracks[0].performer,
            Some("My Bloody Valentine".to_string())
        );
        assert_eq!(cue.files[0].tracks[0].indices.len(), 1);
        assert_eq!(cue.files[0].tracks[0].indices[0], (
            "01".to_string(),
            Duration::new(0, 0),
        ));
    }

    #[test]
    fn test_parsing_unicode() {
        let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
        assert_eq!(
            cue.title,
            Some("マジコカタストロフィ".to_string())
        );
    }

    #[test]
    fn test_case_sensitivity() {
        let cue = parse_from_file("test/fixtures/case_sensitivity.cue", true).unwrap();
        assert_eq!(cue.title, Some("Loveless".to_string()));
        assert_eq!(cue.performer, Some("My Bloody Valentine".to_string()));
    }

    #[test]
    fn test_bad_intentation() {
        let cue = parse_from_file("test/fixtures/bad_indentation.cue", true).unwrap();
        assert_eq!(cue.title, Some("Loveless".to_string()));
        assert_eq!(cue.files.len(), 1);
        assert_eq!(cue.files[0].tracks.len(), 2);
        assert_eq!(
            cue.files[0].tracks[0].title,
            Some("Only Shallow".to_string())
        );
    }

    #[test]
    fn test_unknown_field_lenient() {
        let cue = parse_from_file("test/fixtures/unknown_field.cue", false).unwrap();
        assert_eq!(cue.unknown[0], "FOO WHAT 12345");
    }

    #[test]
    fn test_unknown_field_strict() {
        let cue = parse_from_file("test/fixtures/unknown_field.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_empty_lines_lenient() {
        let cue = parse_from_file("test/fixtures/empty_lines.cue", false).unwrap();
        assert_eq!(cue.comments.len(), 4);
        assert_eq!(cue.files.len(), 1);
        assert_eq!(cue.files[0].tracks.len(), 2);
    }

    #[test]
    fn test_empty_lines_strict() {
        let cue = parse_from_file("test/fixtures/empty_lines.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_duplicate_comment() {
        let cue = parse_from_file("test/fixtures/duplicate_comment.cue", true).unwrap();
        assert_eq!(cue.comments.len(), 5);
        assert_eq!(cue.comments[1], ("DATE".to_string(), "1991".to_string()));
        assert_eq!(cue.comments[2], ("DATE".to_string(), "1992".to_string()));
    }

    #[test]
    fn test_duplicate_title() {
        let cue = parse_from_file("test/fixtures/duplicate_title.cue", true).unwrap();
        assert_eq!(cue.title, Some("Loveless 2".to_string()));
    }

    #[test]
    fn test_duplicate_track() {
        let cue = parse_from_file("test/fixtures/duplicate_track.cue", true).unwrap();
        assert_eq!(cue.files[0].tracks[0], cue.files[0].tracks[1]);
    }

    #[test]
    fn test_duplicate_file() {
        let cue = parse_from_file("test/fixtures/duplicate_file.cue", true).unwrap();
        assert_eq!(cue.files.len(), 2);
        assert_eq!(cue.files[0], cue.files[1]);
    }

    #[test]
    fn test_bad_index_lenient() {
        let cue = parse_from_file("test/fixtures/bad_index.cue", false).unwrap();
        assert_eq!(cue.files[0].tracks[0].indices.len(), 0);
    }

    #[test]
    fn test_bad_index_strict() {
        let cue = parse_from_file("test/fixtures/bad_index.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_bad_index_timestamp_lenient() {
        let cue = parse_from_file("test/fixtures/bad_index_timestamp.cue", false).unwrap();
        assert_eq!(cue.files[0].tracks[0].indices.len(), 0);
    }

    #[test]
    fn test_bad_index_timestamp_strict() {
        let cue = parse_from_file("test/fixtures/bad_index_timestamp.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_pregap_postgap() {
        let cue = parse_from_file("test/fixtures/pregap.cue", true).unwrap();
        assert_eq!(cue.files[0].tracks[0].pregap, Some(Duration::new(1, 0)));
        assert_eq!(cue.files[0].tracks[0].postgap, Some(Duration::new(2, 0)));
    }

    #[test]
    fn test_bad_pregap_timestamp_strict() {
        let cue = parse_from_file("test/fixtures/bad_pregap_timestamp.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_bad_pregap_timestamp_lenient() {
        let cue = parse_from_file("test/fixtures/bad_pregap_timestamp.cue", false).unwrap();
        assert!(cue.files[0].tracks[0].pregap.is_none());
    }

    #[test]
    fn test_bad_postgap_timestamp_strict() {
        let cue = parse_from_file("test/fixtures/bad_postgap_timestamp.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_bad_postgap_timestamp_lenient() {
        let cue = parse_from_file("test/fixtures/bad_postgap_timestamp.cue", false).unwrap();
        assert!(cue.files[0].tracks[0].postgap.is_none());
    }

    #[test]
    fn test_catalog() {
        let cue = parse_from_file("test/fixtures/catalog.cue", true).unwrap();
        assert_eq!(cue.catalog, Some("TESTCATALOG-ID 64".to_string()));
    }

    #[test]
    fn test_comments() {
        let cue = parse_from_file("test/fixtures/comments.cue", true).unwrap();
        assert_eq!(cue.comments.len(), 4);
        assert_eq!(cue.files[0].comments.len(), 1);
        assert_eq!(cue.files[0].tracks[0].comments.len(), 1);
        assert_eq!(cue.files[0].tracks[1].comments.len(), 2);
        assert_eq!(cue.files[0].tracks[1].comments[0], (
            "TRACK".to_string(),
            "2".to_string(),
        ));
        assert_eq!(cue.files[0].tracks[1].comments[1], (
            "TRACK".to_string(),
            "2.1".to_string(),
        ));
    }

    #[test]
    fn test_orphan_track_strict() {
        let cue = parse_from_file("test/fixtures/orphan_track.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_orphan_track_lenient() {
        let cue = parse_from_file("test/fixtures/orphan_track.cue", false).unwrap();
        assert_eq!(cue.files.len(), 0);
    }

    #[test]
    fn test_orphan_index_strict() {
        let cue = parse_from_file("test/fixtures/orphan_index.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_orphan_index_lenient() {
        let cue = parse_from_file("test/fixtures/orphan_index.cue", false).unwrap();
        assert_eq!(cue.files[0].tracks.len(), 1);
        assert_eq!(cue.files[0].tracks[0].indices.len(), 1);
        assert_eq!(cue.files[0].tracks[0].indices[0], (
            "01".to_string(),
            Duration::new(
                257,
                693333333,
            ),
        ));
    }

    #[test]
    fn test_orphan_pregap_strict() {
        let cue = parse_from_file("test/fixtures/orphan_pregap.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_orphan_pregap_lenient() {
        let cue = parse_from_file("test/fixtures/orphan_pregap.cue", false).unwrap();
        assert_eq!(cue.files[0].tracks.len(), 1);
        assert!(cue.files[0].tracks[0].pregap.is_none());
    }

    #[test]
    fn test_orphan_postgap_strict() {
        let cue = parse_from_file("test/fixtures/orphan_postgap.cue", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_orphan_postgap_lenient() {
        let cue = parse_from_file("test/fixtures/orphan_postgap.cue", false).unwrap();
        assert_eq!(cue.files[0].tracks.len(), 1);
        assert!(cue.files[0].tracks[0].pregap.is_none());
    }

    #[test]
    fn test_missing_file() {
        let cue = parse_from_file("test/fixtures/missing.cue.missing", true);
        assert!(cue.is_err());
    }

    #[test]
    fn test_bare_file() {
        use std::io;

        assert!(parse(&mut io::Cursor::new(b"FILE"), true).is_err());
    }
}
