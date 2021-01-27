use crate::markdown::span::parse_spans;
use crate::markdown::Span;
use crate::markdown::Span::Strong;
use fancy_regex::Regex;

/// Parses any strong (bold) markdown tags in the given text.
pub fn parse_strong(text: &str) -> Option<(Span, usize)> {
    lazy_static! {
        static ref STRONG: Regex = Regex::new(r"^\*\*(?P<text>.+?)\*\*(?!\*)").unwrap();
    }

    let mut span = None;
    if let Ok(Some(captures)) = STRONG.captures(text) {
        let t = captures.name("text").expect("no named capture").as_str();
        span = Some((Strong(parse_spans(t)), t.len() + 4));
    }

    span
}

#[cfg(test)]
mod tests {
    use super::parse_strong;
    use super::Span::{Strong, Text};

    #[test]
    fn finds_strong() {
        assert_eq!(
            parse_strong("**this is a** strong string"),
            Some((Strong(vec![Text("this is a".to_owned())]), 13))
        );

        assert_eq!(
            parse_strong("**testing** strong** string"),
            Some((Strong(vec![Text("testing".to_owned())]), 11))
        );
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_strong("** testing string"), None);
        assert_eq!(parse_strong("**** testing string"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_strong("test **test** test"), None);
    }
}
