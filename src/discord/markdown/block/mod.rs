use super::span::parse_spans;
use super::Block;
use super::Block::Paragraph;

mod blockquote;

use self::blockquote::parse_blockquote;

pub fn parse_blocks(content: &str) -> Vec<Block> {
    let mut blocks = vec![];
    let mut t = vec![];

    match parse_block(content) {
        // A block was found
        Some(block) => {
            // The current paragraph has ended, push it to the blocks Vec
            if !t.is_empty() {
                blocks.push(Paragraph(t));
                t = Vec::new();
            }

            blocks.push(block);
        }
        // Didn't find a block, assume it's a Paragraph
        None => {
            // Empty linebreak; push a new Paragraph
            if content.is_empty() && !t.is_empty() {
                blocks.push(Paragraph(t));
                t = Vec::new();
            }

            // Parse any span elements in this line
            let spans = parse_spans(content);
            t.extend_from_slice(&spans);
        }
    }

    if !t.is_empty() {
        blocks.push(Paragraph(t));
    }

    blocks
}

fn parse_block(content: &str) -> Option<Block> {
    pipe_opt!(
        content
        => parse_blockquote
    )
}

#[cfg(test)]
mod tests {
    use super::parse_blocks;
    use super::Block::{Blockquote, Paragraph};
    use crate::markdown::Span::Text;

    #[test]
    fn finds_blockquotes() {
        assert_eq!(
            parse_blocks("> One"),
            vec![Blockquote(vec![Paragraph(vec![Text("One".to_owned())])])]
        )
    }
}
