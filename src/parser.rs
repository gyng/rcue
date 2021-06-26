use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use cue::{Command, Cue, CueFile, Track};
use errors::CueError;
use util::{next_string, next_token, next_values, timestamp_to_duration};

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
    let mut buf_reader = BufReader::new(file);
    parse(&mut buf_reader, strict)
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
/// let mut buf_reader = BufReader::new(file);
/// let cue = parse(&mut buf_reader, true).unwrap();
/// assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
/// ```
///
/// # Failures
///
/// Fails if the CUE file can not be parsed.
#[allow(dead_code)]
pub fn parse(buf_reader: &mut dyn BufRead, strict: bool) -> Result<Cue, CueError> {
    let verbose = env::var_os("RCUE_LOG").map(|s| s == "1").unwrap_or(false);

    macro_rules! fail_if_strict {
        ($line_no:ident, $line:ident, $reason:expr) => {
            if strict {
                if verbose {
                    println!(
                        "Strict mode failure: did not parse line {}: {}\n\tReason: {:?}",
                        $line_no + 1,
                        $line,
                        $reason
                    );
                }
                return Err(CueError::Parse(format!("strict mode failure: {}", $reason)));
            }
        };
    }

    let mut cue = Cue::new();

    fn last_file(cue: &mut Cue) -> Option<&mut CueFile> {
        cue.files.last_mut()
    }

    fn last_track(cue: &mut Cue) -> Option<&mut Track> {
        last_file(cue).and_then(|f| f.tracks.last_mut())
    }

    for (i, line) in buf_reader.lines().enumerate() {
        if let Ok(ref l) = line {
            let token = tokenize_line(l);

            match token {
                Ok(Command::CdTextFile(path)) => {
                    cue.cd_text_file = Some(path);
                }
                Ok(Command::Flags(flags)) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().flags = flags;
                    } else {
                        fail_if_strict!(i, l, "FLAG assigned to no TRACK");
                    }
                }
                Ok(Command::Isrc(isrc)) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().isrc = Some(isrc);
                    } else {
                        fail_if_strict!(i, l, "ISRC assigned to no TRACK");
                    }
                }
                Ok(Command::Rem(field, value)) => {
                    let comment = (field, value);

                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().comments.push(comment);
                    } else if last_file(&mut cue).is_some() {
                        last_file(&mut cue).unwrap().comments.push(comment);
                    } else {
                        cue.comments.push(comment);
                    }
                }
                Ok(Command::File(file, format)) => {
                    cue.files.push(CueFile::new(&file, &format));
                }
                Ok(Command::Track(idx, mode)) => {
                    if let Some(file) = last_file(&mut cue) {
                        file.tracks.push(Track::new(&idx, &mode));
                    } else {
                        fail_if_strict!(i, l, "TRACK assigned to no FILE");
                    }
                }
                Ok(Command::Title(title)) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().title = Some(title);
                    } else {
                        cue.title = Some(title)
                    }
                }
                Ok(Command::Performer(performer)) => {
                    // this double check might be able to go away under non-lexical lifetimes
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().performer = Some(performer);
                    } else {
                        cue.performer = Some(performer);
                    }
                }
                Ok(Command::Songwriter(songwriter)) => {
                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().songwriter = Some(songwriter);
                    } else {
                        cue.songwriter = Some(songwriter);
                    }
                }
                Ok(Command::Index(idx, time)) => {
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
                Ok(Command::Pregap(time)) => {
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
                Ok(Command::Postgap(time)) => {
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
                Ok(Command::Catalog(id)) => {
                    cue.catalog = Some(id);
                }
                Ok(Command::Unknown(line)) => {
                    fail_if_strict!(i, l, &format!("unknown token -- {}", &line));

                    if last_track(&mut cue).is_some() {
                        last_track(&mut cue).unwrap().unknown.push(line);
                    } else {
                        cue.unknown.push(line)
                    }
                }
                _ => {
                    fail_if_strict!(i, l, &format!("bad line -- {:?}", &line));
                    if verbose {
                        println!("Bad line - did not parse line {}: {:?}", i + 1, l);
                    }
                }
            }
        }
    }

    Ok(cue)
}

#[allow(dead_code)]
fn tokenize_line(line: &str) -> Result<Command, CueError> {
    let mut chars = line.trim().chars();

    let command = next_token(&mut chars);
    let command = if command.is_empty() {
        None
    } else {
        Some(command)
    };

    match command {
        Some(c) => match c.to_uppercase().as_ref() {
            "REM" => {
                let key = next_token(&mut chars);
                let val = next_string(&mut chars, "missing REM value")?;
                Ok(Command::Rem(key, val))
            }
            "CATALOG" => {
                let val = next_string(&mut chars, "missing CATALOG value")?;
                Ok(Command::Catalog(val))
            }
            "CDTEXTFILE" => {
                let val = next_string(&mut chars, "missing CDTEXTFILE value")?;
                Ok(Command::CdTextFile(val))
            }
            "TITLE" => {
                let val = next_string(&mut chars, "missing TITLE value")?;
                Ok(Command::Title(val))
            }
            "FILE" => {
                let path = next_string(&mut chars, "missing path for FILE")?;
                let format = next_token(&mut chars);
                Ok(Command::File(path, format))
            }
            "FLAGS" => {
                let flags = next_values(&mut chars);
                Ok(Command::Flags(flags))
            }
            "ISRC" => {
                let val = next_token(&mut chars);
                Ok(Command::Isrc(val))
            }
            "PERFORMER" => {
                let val = next_string(&mut chars, "missing PERFORMER value")?;
                Ok(Command::Performer(val))
            }
            "SONGWRITER" => {
                let val = next_string(&mut chars, "missing SONGWRITER value")?;
                Ok(Command::Songwriter(val))
            }
            "TRACK" => {
                let val = next_token(&mut chars);
                let mode = next_token(&mut chars);
                Ok(Command::Track(val, mode))
            }
            "PREGAP" => {
                let val = next_token(&mut chars);
                Ok(Command::Pregap(val))
            }
            "POSTGAP" => {
                let val = next_token(&mut chars);
                Ok(Command::Postgap(val))
            }
            "INDEX" => {
                let val = next_token(&mut chars);
                let time = next_token(&mut chars);
                Ok(Command::Index(val, time))
            }
            _ => {
                let rest: String = chars.collect();
                if rest.is_empty() {
                    Ok(Command::None)
                } else {
                    Ok(Command::Unknown(line.to_string()))
                }
            }
        },
        _ => Ok(Command::None),
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
        assert_eq!(
            cue.comments[0],
            ("GENRE".to_string(), "Alternative".to_string(),)
        );
        assert_eq!(cue.comments[1], ("DATE".to_string(), "1991".to_string()));
        assert_eq!(
            cue.comments[2],
            ("DISCID".to_string(), "860B640B".to_string(),)
        );
        assert_eq!(
            cue.comments[3],
            ("COMMENT".to_string(), "ExactAudioCopy v0.95b4".to_string(),)
        );
        assert_eq!(cue.performer, Some("My Bloody Valentine".to_string()));
        assert_eq!(cue.songwriter, Some("foobar".to_string()));
        assert_eq!(cue.title, Some("Loveless".to_string()));
        assert_eq!(cue.cd_text_file, Some("./cdtextfile".to_string()));

        assert_eq!(cue.files.len(), 1);
        let ref file = cue.files[0];
        assert_eq!(file.file, "My Bloody Valentine - Loveless.wav");
        assert_eq!(file.format, "WAVE");

        assert_eq!(file.tracks.len(), 2);
        let ref track = file.tracks[0];
        assert_eq!(track.no, "01".to_string());
        assert_eq!(track.format, "AUDIO".to_string());
        assert_eq!(track.songwriter, Some("barbaz bax".to_string()));
        assert_eq!(track.title, Some("Only Shallow".to_string()));
        assert_eq!(track.performer, Some("My Bloody Valentine".to_string()));
        assert_eq!(track.indices.len(), 1);
        assert_eq!(track.indices[0], ("01".to_string(), Duration::new(0, 0)));
        assert_eq!(track.isrc, Some("USRC17609839".to_string()));
        assert_eq!(track.flags, vec!["DCP", "4CH", "PRE", "SCMS"]);
    }

    #[test]
    fn test_parsing_unicode() {
        let cue = parse_from_file("test/fixtures/unicode.cue", true).unwrap();
        assert_eq!(cue.title, Some("マジコカタストロフィ".to_string()));
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
        assert_eq!(
            cue.files[0].tracks[1].comments[0],
            ("TRACK".to_string(), "2".to_string(),)
        );
        assert_eq!(
            cue.files[0].tracks[1].comments[1],
            ("TRACK".to_string(), "2.1".to_string(),)
        );
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
        assert_eq!(
            cue.files[0].tracks[0].indices[0],
            ("01".to_string(), Duration::new(257, 693333333,),)
        );
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
