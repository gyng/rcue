use std::str::Chars;
use std::time::Duration;

use errors::CueError;

/// Unescapes a string in a CUE field.
///
/// Strings in a CUE field are delimited by double quotation marks (`"`).
///
/// ```text
/// // Example line from a CUE
/// TITLE "My \"Cute\" Song Title"
/// ```
///
/// # Example
///
/// ```
/// use rcue::util::unescape_quotes;
///
/// let unescaped = unescape_quotes(r#""lmao \"i\"cons""#);
/// let expected = r#"lmao "i"cons"#;
/// assert_eq!(unescaped, expected);
/// ```
pub fn unescape_quotes(s: &str) -> String {
    let mut unescaped = s.replace("\\\"", "\"");

    if unescaped.ends_with("\"") && unescaped.starts_with("\"") {
        unescaped.pop();
        unescaped.remove(0);
    }

    unescaped
}

/// Converts a CUE timestamp (MM:SS:FF) to a
/// [Duration](https://doc.rust-lang.org/nightly/std/time/duration/struct.Duration.html)
/// where each frame FF is `1 / 75` of a second.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use rcue::util::timestamp_to_duration;
///
/// let duration = timestamp_to_duration("99:99:99").unwrap();
/// assert_eq!(duration, Duration::new(6040, 320000000));
/// ```
///
/// # Failures
///
/// Fails if timestamp is not valid
#[allow(dead_code)]
pub fn timestamp_to_duration(s: &str) -> Result<Duration, CueError> {
    fn next_group(chars: &mut Chars) -> String {
        chars.take_while(|c| *c != ':').collect::<String>()
    }

    let timestamp = s.to_string();
    let mut iter = timestamp.chars();
    let minutes: String = next_group(&mut iter);
    let seconds: String = next_group(&mut iter);
    let frames: String = iter.collect();

    let frame_seconds = frames.parse::<f64>()? / 75.0;
    let seconds = minutes.parse::<u64>()? * 60 + seconds.parse::<u64>()? +
        frame_seconds.floor() as u64;
    let nanos = (frame_seconds.fract() * 1000000000f64) as u32;

    Ok(Duration::new(seconds, nanos))
}

/// Returns the next token from a [`Chars`](https://doc.rust-lang.org/std/str/struct.Chars.html).
/// This does *not* ignore leading whitespace.
///
/// # Example
///
/// ```
/// use rcue::util::next_token;
///
/// let tokens = "a b c d".to_string();
/// let mut chars = tokens.chars();
/// assert_eq!(next_token(&mut chars), "a".to_string());
/// assert_eq!(next_token(&mut chars), "b".to_string());
/// ```
pub fn next_token(chars: &mut Chars) -> String {
    let token = chars.take_while(|c| !c.is_whitespace()).collect::<String>();
    token
}

/// Returns the next bare (single-word) or quoted (single- or multi-word)
/// [`String`](https://doc.rust-lang.org/std/string/struct.String.html).
/// This does *not* ignore leading whitespace.
///
/// # Example
///
/// ```
/// use rcue::util::next_string;
///
/// let quotes = r#""quotation \"\" marks""#.to_string();
/// let actual = next_string(&mut quotes.chars(), "").unwrap();
/// let expected = r#"quotation "" marks"#;
/// assert_eq!(actual, expected);
/// ```
///
/// # Failures
///
/// Fails if no string can be parsed (eg. an unexpected EOL)
#[allow(dead_code)]
pub fn next_string(chars: &mut Chars, error: &str) -> Result<String, CueError> {
    let first = chars.next().ok_or(CueError::Parse(error.to_string()))?;

    if first == '"' {
        let mut escaped = false;
        let string = chars
            .take_while(|c| {
                if !escaped && *c == '\\' {
                    println!("turning on escape");
                    escaped = true;
                    return true;
                }

                if escaped {
                    escaped = false;
                    return true;
                }

                *c != '"'
            })
            .collect::<String>();
        let _next_space = chars.next().ok_or(CueError::Parse(
            "Unexpected error: could not consume next space. This is likely a bug."
                .to_string(),
        ));

        Ok(unescape_quotes(&string))
    } else {
        let string = first.to_string() + &next_token(chars);

        Ok(unescape_quotes(&string))
    }
}

/// Returns a list of values, split by whitespace.
///
/// # Example
///
/// ```
/// use rcue::util::next_values;
///
/// let values = "a b".to_string();
/// let mut iter = values.chars();
/// let actual = next_values(&mut iter);
/// let expected = vec!["a".to_string(), "b".to_string()];
/// assert_eq!(actual, expected);
/// ```
#[allow(dead_code)]
pub fn next_values(chars: &mut Chars) -> Vec<String> {
    let string: String = chars.collect();
    string.split_whitespace().map(|s| s.to_string()).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_unescape_quotes() {
        let actual = unescape_quotes(r#""lmao \"i\"cons""#);
        let expected = r#"lmao "i"cons"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_unescape_unescaped_string() {
        let actual = unescape_quotes(r#"lmao "i"cons"#);
        let expected = r#"lmao "i"cons"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_valid_timestamp_conversion() {
        let actual = timestamp_to_duration("00:00:00").unwrap();
        let expected = Duration::new(0, 0);
        assert_eq!(actual, expected);

        let actual = timestamp_to_duration("01:01:00").unwrap();
        let expected = Duration::new(61, 0);
        assert_eq!(actual, expected);

        let actual = timestamp_to_duration("99:99:99").unwrap();
        let expected = Duration::new(6040, 320000000);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_frame_second_conversion() {
        let actual = timestamp_to_duration("00:00:75").unwrap();
        let expected = Duration::new(1, 0);
        assert_eq!(actual, expected);

        let actual = timestamp_to_duration("00:00:76").unwrap();
        let expected = Duration::new(1, 13333333);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_invalid_timestamp() {
        assert!(timestamp_to_duration("000000").is_err());
        assert!(timestamp_to_duration("-00:00:00").is_err());
        assert!(timestamp_to_duration("00:00").is_err());
        assert!(timestamp_to_duration("00 00 00").is_err());
        assert!(timestamp_to_duration("00.00.00").is_err());
        assert!(timestamp_to_duration(" 00:00:00").is_err());
        assert!(timestamp_to_duration("00:00:00 ").is_err());
        assert!(timestamp_to_duration(" 00:00:00 ").is_err());
        assert!(timestamp_to_duration("P0003-06-04T12:30:05").is_err());
    }

    #[test]
    fn test_next_string_quotation_marks() {
        let quotes = r#""quotation \"\" marks""#.to_string();
        let actual = next_string(&mut quotes.chars(), "").unwrap();
        let expected = r#"quotation "" marks"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_next_string_single_quotation_marks() {
        let quotes_single = r#"this\"isfine"#.to_string();
        let actual = next_string(&mut quotes_single.chars(), "").unwrap();
        let expected = r#"this"isfine"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_next_tokens() {
        let tokens = "a b c d".to_string();
        let mut iter = tokens.chars();
        assert_eq!(next_token(&mut iter), "a".to_string());
        assert_eq!(next_token(&mut iter), "b".to_string());
        assert_eq!(next_token(&mut iter), "c".to_string());
        assert_eq!(next_token(&mut iter), "d".to_string());
    }

    #[test]
    fn test_next_values() {
        let values = "a b".to_string();
        let mut iter = values.chars();
        let actual = next_values(&mut iter);
        let expected = vec!["a".to_string(), "b".to_string()];
        assert_eq!(actual, expected);
    }
}
