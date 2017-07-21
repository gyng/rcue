use regex::Regex;

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
/// use rcue::util::unescape_string;
///
/// let unescaped = unescape_string(r#""lmao \"i\"cons""#);
/// let expected = r#"lmao "i"cons"#;
/// assert_eq!(unescaped, expected);
/// ```
#[allow(dead_code)]
pub fn unescape_string(s: &str) -> String {
    lazy_static! {
        static ref START_END_DOUBLE_QUOTE: Regex = Regex::new(r#"(^"|"$)"#).unwrap();
        static ref ESCAPED_DOUBLE_QUOTE: Regex = Regex::new(r#"\\""#).unwrap();
    }

    let escaped = START_END_DOUBLE_QUOTE.replace_all(s, "").to_string();
    let escaped = ESCAPED_DOUBLE_QUOTE.replace_all(&escaped, "\"").to_string();

    escaped
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
    lazy_static! {
        static ref TIMESTAMP: Regex = Regex::new(r#"^(?P<m>\d\d):(?P<s>\d\d):(?P<f>\d\d)$"#).unwrap();
    }

    if !TIMESTAMP.is_match(s) {
        return Err(CueError::Parse(format!("Invalid timestamp format {}", s)));
    }

    let captures = TIMESTAMP.captures(s).ok_or_else(|| {
        CueError::Parse(format!("invalid timestamp format: {}", s))
    })?;

    let minutes = &captures["m"];
    let seconds = &captures["s"];
    let frames = &captures["f"]; // there are 75 frames to one second

    let frame_seconds = frames.parse::<f64>()? / 75.0;
    let seconds = minutes.parse::<u64>()? * 60 + seconds.parse::<u64>()? +
        frame_seconds.floor() as u64;
    let nanos = (frame_seconds.fract() * 1000000000f64) as u32;

    Ok(Duration::new(seconds, nanos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_unescape_string() {
        let actual = unescape_string(r#""lmao \"i\"cons""#);
        let expected = r#"lmao "i"cons"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_unescape_unescaped_string() {
        let actual = unescape_string(r#"lmao "i"cons"#);
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
}
