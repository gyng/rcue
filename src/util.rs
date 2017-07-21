use regex::Regex;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_string() {
        let actual = unescape_string(r#""lmao \"i\"cons""#);
        let expected = r#"lmao "i"cons"#;
        assert_eq!(actual, expected);
    }
}
