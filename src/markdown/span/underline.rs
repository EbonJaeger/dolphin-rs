use crate::markdown::span::parse_spans;
use crate::markdown::Span;
use crate::markdown::Span::Underline;
use regex::Regex;

/// Parses any underline markdown tags in the given text.
pub fn parse_underline(text: &str) -> Option<(Span, usize)> {
    lazy_static! {
        static ref EMPHASIS: Regex = Regex::new(r"^__(?P<text>.+?)__").unwrap();
    }

    if EMPHASIS.is_match(text) {
        let captures = EMPHASIS.captures(text).unwrap();
        let t = captures.name("text").unwrap().as_str();
        Some((Underline(parse_spans(t)), t.len() + 4))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_underline;
    use super::Span::{Text, Underline};

    #[test]
    fn finds_underline() {
        assert_eq!(
            parse_underline("__this is an__ underlined string"),
            Some((Underline(vec![Text("this is an".to_owned())]), 14))
        );

        assert_eq!(
            parse_underline("__testing__ underlined__ strings"),
            Some((Underline(vec![Text("testing".to_owned())]), 11))
        );
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_underline("__ testing string"), None);
        assert_eq!(parse_underline("____ testing string"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_underline("test __test__ test"), None);
    }
}
