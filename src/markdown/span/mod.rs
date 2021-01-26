use super::Span;
use super::Span::{Literal, Text};

mod emphasis;
mod strikethrough;
mod strong;
mod underline;

use self::emphasis::parse_emphasis;
use self::strikethrough::parse_strikethrough;
use self::strong::parse_strong;
use self::underline::parse_underline;

/// Parses a piece of text for span-type elements, returning the whole thing in
/// a Vector tree.
pub fn parse_spans(content: &str) -> Vec<Span> {
    let mut tokens = vec![];
    let mut t = String::new();
    let mut index = 0;

    while index < content.len() {
        match parse_span(&content[index..content.len()]) {
            // Found a span element
            Some((span, consumed)) => {
                if !t.is_empty() {
                    // This token is on the far left, so trim left whitespace
                    if tokens.is_empty() {
                        t = t.trim_start().to_owned()
                    }
                    // Put the text for this element inside the span
                    tokens.push(Text(t));
                }

                tokens.push(span);
                t = String::new();
                index += consumed;
            }
            // No span elements found, so push the rest of the content
            None => {
                let mut end = index + 1;
                while !content.is_char_boundary(end) {
                    end += 1;
                }

                t.push_str(&content[index..end]);
                index += end - index;
            }
        }
    }

    if !t.is_empty() {
        // Trim whitespaces
        if tokens.is_empty() {
            t = t.trim_start().to_owned()
        }
        t = t.trim_end().to_owned();

        tokens.push(Text(t));
    }

    tokens
}

fn parse_escape(content: &str) -> Option<(Span, usize)> {
    let mut chars = content.chars();
    if let Some('\\') = chars.next() {
        return match chars.next() {
            Some(x @ '\\') | Some(x @ '`') | Some(x @ '*') | Some(x @ '_') => Some((Literal(x), 2)),
            _ => None,
        };
    }

    None
}

fn parse_span(content: &str) -> Option<(Span, usize)> {
    pipe_opt!(
        content
        => parse_escape
        => parse_strong
        => parse_emphasis
        => parse_strikethrough
        => parse_underline
    )
}
