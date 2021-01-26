use crate::markdown::span::parse_spans;
use crate::markdown::Span;
use crate::markdown::Span::Emphasis;
use regex::Regex;

/// Parses any emphasis (italic) markdown tags in the given text.
pub fn parse_emphasis(text: &str) -> Option<(Span, usize)> {
    lazy_static! {
        static ref EMPHASIS: Regex = Regex::new(r"^\*(?P<text>.+?)\*").unwrap();
    }

    if EMPHASIS.is_match(text) {
        let captures = EMPHASIS.captures(text).unwrap();
        let t = captures.name("text").unwrap().as_str();
        Some((Emphasis(parse_spans(t)), t.len() + 2))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_emphasis;
    use super::Span::{Emphasis, Text};

    #[test]
    fn finds_emphasis() {
        assert_eq!(
            parse_emphasis("*this is a* strong string"),
            Some((Emphasis(vec![Text("this is a".to_owned())]), 11))
        );

        assert_eq!(
            parse_emphasis("*testing* strong* string"),
            Some((Emphasis(vec![Text("testing".to_owned())]), 9))
        );
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_emphasis("* testing string"), None);
        assert_eq!(parse_emphasis("** testing string"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_emphasis("test *test* test"), None);
    }
}
