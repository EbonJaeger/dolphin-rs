use super::Block;
use super::Block::Blockquote;
use crate::discord::markdown::block::parse_blocks;

pub fn parse_blockquote(line: &str) -> Option<Block> {
    if line.is_empty() || !line.starts_with("> ") {
        return None;
    }

    let mut content = String::new();

    // Push the content of the quote after the opening `>`
    content.push_str(&line[2..line.len()]);

    Some(Blockquote(parse_blocks(&content)))
}

#[cfg(test)]
mod tests {
    use super::parse_blockquote;
    use super::Block::Blockquote;

    #[test]
    fn finds_blockquote() {
        match parse_blockquote("> quote") {
            Some(Blockquote(_)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_blockquote(">shouldn't parse"), None);
        assert_eq!(parse_blockquote("shouldn't > parse"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_blockquote("first > quote > another blah"), None);
    }
}
