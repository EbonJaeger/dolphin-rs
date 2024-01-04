use crate::discord::markdown::span::parse_spans;
use crate::discord::markdown::Span;
use crate::discord::markdown::Span::Emphasis;
use fancy_regex::Regex;

/// Parses any emphasis (italic) markdown tags in the given text.
///
/// # Regex
///
/// This thing uses a monster regex from the `simple-markdown` Github project
/// found [here](https://github.com/Khan/simple-markdown/blob/master/src/index.js#L1607):
///
/// ```js
/// match: inlineRegex(
///     new RegExp(
///         // only match _s surrounding words.
///         "^\\b_" +
///         "((?:__|\\\\[\\s\\S]|[^\\\\_])+?)_" +
///         "\\b" +
///         // Or match *s:
///         "|" +
///         // Only match *s that are followed by a non-space:
///         "^\\*(?=\\S)(" +
///         // Match at least one of:
///         "(?:" +
///           //  - `**`: so that bolds inside italics don't close the
///           //          italics
///           "\\*\\*|" +
///           //  - escape sequence: so escaped *s don't close us
///           "\\\\[\\s\\S]|" +
///           //  - whitespace: followed by a non-* (we don't
///           //          want ' *' to close an italics--it might
///           //          start a list)
///           "\\s+(?:\\\\[\\s\\S]|[^\\s\\*\\\\]|\\*\\*)|" +
///           //  - non-whitespace, non-*, non-backslash characters
///           "[^\\s\\*\\\\]" +
///         ")+?" +
///         // followed by a non-space, non-* then *
///         ")\\*(?!\\*)"
///       )
///   )
/// ```
pub fn parse_emphasis(text: &str) -> Option<(Span, usize)> {
    // Slight hack so I don't have to spend any more time
    // in Regex Hell.
    if text.starts_with("***") {
        return None;
    }

    lazy_static! {
        static ref EMPHASIS: Regex = Regex::new(r"^\b_((?:__|\\[\s\S]|[^\\_])+?)_\b|^\*(?=\S)((?:\*\*|\\[\s\S]|\s+(?:\\[\s\S]|[^\s\*\\]|\*\*)|[^\s\*\\])+?)\*(?!\*)").unwrap();
    }

    let mut span = None;
    if let Ok(Some(captures)) = EMPHASIS.captures(text) {
        // Look for one form of emphasis tag, then look for the other if
        // that isn't found.
        let t = match captures.get(2) {
            Some(m) => m.as_str(),
            None => match captures.get(1) {
                Some(m) => m.as_str(),
                None => panic!("emphasis regex found matches, but couldn't get capture groups"),
            },
        };

        span = Some((Emphasis(parse_spans(t)), t.len() + 2));
    }

    span
}

#[cfg(test)]
mod tests {
    use super::parse_emphasis;
    use super::Span::{Emphasis, Text};

    #[test]
    fn finds_emphasis() {
        assert_eq!(
            parse_emphasis("*this is an* italic string"),
            Some((Emphasis(vec![Text("this is an".to_owned())]), 12))
        );

        assert_eq!(
            parse_emphasis("*testing* italic* string"),
            Some((Emphasis(vec![Text("testing".to_owned())]), 9))
        );

        assert_eq!(
            parse_emphasis("_this is also_ an italic string"),
            Some((Emphasis(vec![Text("this is also".to_owned())]), 14))
        );
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_emphasis("* testing string"), None);
        assert_eq!(parse_emphasis("** testing string"), None);

        assert_eq!(parse_emphasis("_ testing string"), None);
        assert_eq!(parse_emphasis("__ testing string"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_emphasis("test *test* test"), None);

        assert_eq!(parse_emphasis("test _test_ test"), None);
    }
}
