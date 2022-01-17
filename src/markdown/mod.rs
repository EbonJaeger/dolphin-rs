mod block;
mod span;

const EMPHASIS_TAG: &str = "§o";
const RESET_TAG: &str = "§r";
const STRIKETHROUGH_TAG: &str = "§m";
const STRONG_TAG: &str = "§l";
const UNDERLINE_TAG: &str = "§n";

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

impl Span {
    /// Converts a single Span element (and any nested Spans, recursively)
    /// to a String with the Minecraft format tags.
    ///
    /// Minecraft doesn't have real closing tags; you have to completely
    /// reset the string and re-apply tags that should still be applied.
    /// Thus, we need to keep track of any open tags with a `Vec<String>`.
    fn to_minecraft(&self, open_tags: &mut Vec<String>) -> String {
        match self {
            Span::Literal(ref c) => c.to_string(),
            Span::Text(ref content) => content.to_string(),
            Span::Emphasis(ref content) => {
                open_tags.push(EMPHASIS_TAG.to_owned());
                let span = format_spans(content, open_tags);
                format!("{}{}{}", EMPHASIS_TAG, span, RESET_TAG)
            }
            Span::Strong(ref content) => {
                open_tags.push(STRONG_TAG.to_owned());
                let span = format_spans(content, open_tags);
                format!("{}{}{}", STRONG_TAG, span, RESET_TAG)
            }
            Span::Strikethrough(ref content) => {
                open_tags.push(STRIKETHROUGH_TAG.to_owned());
                let span = format_spans(content, open_tags);
                format!("{}{}{}", STRIKETHROUGH_TAG, span, RESET_TAG)
            }
            Span::Underline(ref content) => {
                open_tags.push(UNDERLINE_TAG.to_owned());
                let span = format_spans(content, open_tags);
                format!("{}{}{}", UNDERLINE_TAG, span, RESET_TAG)
            }
        }
    }
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

/// Turn a Span tree to a String in the Minecraft chat format. This
/// requires a `Vec<String>` to track any open tags for nested spans
/// because of how formatting in Minecraft works.
///
/// Minecraft doesn't have real closing tags; you have to completely
/// reset the string and re-apply tags that should still be applied.
fn format_spans(elements: &[Span], open_tags: &mut Vec<String>) -> String {
    let mut ret = String::new();

    for element in elements.iter() {
        // Append the element to the final String
        let next = element.to_minecraft(open_tags);
        ret.push_str(&next);

        // Check if we need to add any open tags
        if !open_tags.is_empty() && ret.ends_with(RESET_TAG) {
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
    fn formats_blockquotes() {
        let input = "> blockquote";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "> blockquote");

        let input = "> *blockquote*";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "> §oblockquote§r");
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

    #[test]
    fn escapes_characters() {
        let input = "\\*test\\*";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "*test*");

        let input = "\\*\\*test\\*\\*";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "**test**");

        let input = "\\_test\\_";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "_test_");

        let input = "\\_\\_test\\_\\_";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "__test__");

        let input = "\\~\\~test\\~\\~";
        let md = parse(input);
        assert_eq!(to_minecraft_format(&md), "~~test~~");
    }
}
