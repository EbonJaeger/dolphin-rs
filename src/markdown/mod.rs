mod block;
mod span;

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Paragraph(Vec<Span>),
    Blockquote(Vec<Block>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    Emphasis(Vec<Span>),
    Literal(char),
    Strikethrough(Vec<Span>),
    Strong(Vec<Span>),
    Text(String),
    Underline(Vec<Span>),
}

/// Parse a string and turn it into a tree of Markdown elements.
pub fn parse(content: &str) -> Vec<Block> {
    block::parse_blocks(content)
}

/// Turn a tree of Markdown blocks into a Minecraft formatted string.
pub fn to_minecraft_format(blocks: &[Block]) -> String {
    let mut ret = String::new();

    for block in blocks {
        let next = match block {
            Block::Paragraph(ref elements) => format_paragraph(elements),
            Block::Blockquote(ref elements) => format_blockquote(elements),
        };

        ret.push_str(&next);
    }

    ret
}

fn format_blockquote(elements: &[Block]) -> String {
    format!("> {}", to_minecraft_format(elements))
}

fn format_paragraph(elements: &[Span]) -> String {
    format_spans(elements, &mut vec![])
}

fn format_spans(elements: &[Span], mut open_tags: &mut Vec<String>) -> String {
    let mut ret = String::new();

    for element in elements.iter() {
        let next = match *element {
            Span::Literal(ref c) => c.to_string(),
            Span::Text(ref content) => content.to_string(),
            Span::Emphasis(ref content) => {
                open_tags.push("§o".to_owned());
                format!("§o{}§r", format_spans(content, &mut open_tags))
            }
            Span::Strong(ref content) => {
                open_tags.push("§l".to_owned());
                format!("§l{}§r", format_spans(content, &mut open_tags))
            }
            Span::Strikethrough(ref content) => {
                open_tags.push("§m".to_owned());
                format!("§m{}§r", format_spans(content, &mut open_tags))
            }
            Span::Underline(ref content) => {
                open_tags.push("§n".to_owned());
                format!("§n{}§r", format_spans(content, &mut open_tags))
            }
        };

        // Append the element to the final String
        ret.push_str(&next);

        // Check if we need to add any open tags
        if open_tags.len() > 0 && ret.ends_with("§r") {
            open_tags.pop();
            ret.push_str(&open_tags.concat());
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::parse;
    use super::to_minecraft_format;

    #[test]
    fn formats_regular_text() {
        let input = "test";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "test");
    }

    #[test]
    fn handles_mixed_emphasis_and_strong() {
        let input = "***test* test**";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§l§otest§r§l test§r");
    }

    #[test]
    fn handles_mixed_underline_and_strikethrough() {
        let input = "__test ~~test~~__";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§ntest §mtest§r§n§r");
    }

    #[test]
    fn handles_mixed_strikethrough_and_underline() {
        let input = "~~test __test__~~";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§mtest §ntest§r§m§r");
    }
}
