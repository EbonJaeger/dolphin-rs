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
    fn handles_emphasis_inner_strong() {
        let input = "*em **strong***";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§oem §lstrong§r§o§r");
    }

    #[test]
    fn handles_strong_inner_emphasis() {
        let input = "**strong *em***";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§lstrong §oem§r§l§r");
    }

    #[test]
    fn handles_strong_inner_emphasis2() {
        let input = "***em* strong**";
        let md = parse(input);
        println!("Blocks: {:?}", md);
        assert_eq!(to_minecraft_format(&md), "§l§oem§r§l strong§r");
    }

    #[test]
    fn handles_emphasis_inner_underline() {
        let input = "___underline__ em_";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§o§nunderline§r§o em§r");
    }

    #[test]
    fn handles_emphasis_inner_underline2() {
        let input = "_em __underline___";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§oem §nunderline§r§o§r");
    }

    #[test]
    fn handles_underline_inner_emphasis() {
        let input = "__underline _em___";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "§nunderline §oem§r§n§r");
    }

    #[test]
    fn handles_underline_inner_strikethrough() {
        let input = "__underline ~~strikethrough~~__";
        let md = parse(input);
        assert_eq!(
            to_minecraft_format(&md),
            "§nunderline §mstrikethrough§r§n§r"
        );
    }

    #[test]
    fn handles_underline_inner_strikethrough2() {
        let input = "__~~strikethrough~~ underline__";
        let md = parse(input);
        assert_eq!(
            to_minecraft_format(&md),
            "§n§mstrikethrough§r§n underline§r"
        );
    }

    #[test]
    fn handles_strikethrough_inner_underline() {
        let input = "~~strikethrough __underline__~~";
        let md = parse(input);
        assert_eq!(
            to_minecraft_format(&md),
            "§mstrikethrough §nunderline§r§m§r"
        );
    }

    #[test]
    fn handles_strikethrough_inner_underline2() {
        let input = "~~__underline__ strikethrough~~";
        let md = parse(input);
        assert_eq!(
            to_minecraft_format(&md),
            "§m§nunderline§r§m strikethrough§r"
        );
    }
}
