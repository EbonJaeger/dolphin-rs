use super::Block;
use super::Block::Blockquote;
use crate::markdown::block::parse_blocks;

pub fn parse_blockquote(lines: &[&str]) -> Option<(Block, usize)> {
    if lines[0].is_empty() || !lines[0].starts_with('>') {
        return None;
    }

    let mut content = String::new();
    let mut index = 0;
    let mut prev_newline = false;

    for line in lines {
        if prev_newline && !line.is_empty() && !line.starts_with('>') {
            break;
        }
        if line.is_empty() {
            prev_newline = true;
        } else {
            prev_newline = false;
        }

        let mut chars = line.chars();
        let start = match chars.next() {
            Some('>') => match chars.next() {
                Some(' ') => 2,
                _ => 1,
            },
            _ => 0,
        };

        if index > 0 {
            content.push('\n');
        }

        content.push_str(&line[start..line.len()]);
        index += 1;
    }

    if index > 0 {
        Some((Blockquote(parse_blocks(&content)), index))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_blockquote;
    use super::Block::Blockquote;

    #[test]
    fn finds_blockquote() {
        match parse_blockquote(&vec!["> quote", "> another quote"]) {
            Some((Blockquote(_), 2)) => (),
            _ => panic!(),
        }

        match parse_blockquote(&vec!["> quote", "> another quote", "blah blah"]) {
            Some((Blockquote(_), 3)) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn stops_parsing_correctly() {
        match parse_blockquote(&vec!["> quote", "> more", "", "blah"]) {
            Some((Blockquote(_), 3)) => (),
            _ => panic!("did not stop parsing"),
        }
    }

    #[test]
    fn no_false_positives() {
        assert_eq!(parse_blockquote(&vec!["shouldn't > parse"]), None);
    }

    #[test]
    fn no_early_matching() {
        assert_eq!(
            parse_blockquote(&vec!["first", "> quote", "> another", "", "blah"]),
            None
        );
    }
}
