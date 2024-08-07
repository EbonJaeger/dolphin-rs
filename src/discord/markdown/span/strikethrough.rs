use crate::discord::markdown::span::parse_spans;
use crate::discord::markdown::Span;
use crate::discord::markdown::Span::Strikethrough;
use fancy_regex::Regex;

/// Parses any strikethrough markdown tags in the given text.
pub fn parse_strikethrough(text: &str) -> Option<(Span, usize)> {
    lazy_static! {
        static ref STRIKETHROUGH: Regex = Regex::new(r"^~~(?P<text>.+?)~~").unwrap();
    }

    let mut span = None;
    if let Ok(Some(captures)) = STRIKETHROUGH.captures(text) {
        let t = captures.name("text").expect("no named capture").as_str();
        span = Some((Strikethrough(parse_spans(t)), t.len() + 4));
    }

    span
}

#[cfg(test)]
mod tests {
    use super::parse_strikethrough;
    use super::Span::{Strikethrough, Text};

    #[test]
    fn finds_strikethrough() {
        assert_eq!(
            parse_strikethrough("~~this is a~~ strong string"),
            Some((Strikethrough(vec![Text("this is a".to_owned())]), 13))
        );

        assert_eq!(
            parse_strikethrough("~~testing~~ strong~~ string"),
            Some((Strikethrough(vec![Text("testing".to_owned())]), 11))
        );
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_strikethrough("~~ testing string"), None);
        assert_eq!(parse_strikethrough("~~~~ testing string"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_strikethrough("test ~~test~~ test"), None);
    }
}
