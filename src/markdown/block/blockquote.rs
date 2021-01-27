use super::Block;
use super::Block::Blockquote;
use crate::markdown::block::parse_blocks;

pub fn parse_blockquote(line: &str) -> Option<Block> {
    if line.is_empty() || !line.starts_with('>') {
        return None;
    }

    let mut content = String::new();
    let mut chars = line.chars();
    let start = match chars.next() {
        Some('>') => match chars.next() {
            Some(' ') => 2,
            _ => 1,
        },
        _ => 0,
    };

    content.push_str(&line[start..line.len()]);

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
        assert_eq!(parse_blockquote("shouldn't > parse"), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(parse_blockquote("first > quote > another blah"), None);
    }
}
