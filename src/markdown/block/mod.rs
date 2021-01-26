use super::span::parse_spans;
use super::Block;
use super::Block::Paragraph;
use super::Span::Text;

mod blockquote;

use self::blockquote::parse_blockquote;

pub fn parse_blocks(content: &str) -> Vec<Block> {
    let mut blocks = vec![];
    let mut t = vec![];
    let lines: Vec<&str> = content.lines().collect();
    let mut index = 0;

    while index < lines.len() {
        match parse_block(&lines[index..lines.len()]) {
            // A block was found
            Some((block, consumed)) => {
                // The current paragraph has ended, push it to the blocks Vec
                if !t.is_empty() {
                    blocks.push(Paragraph(t));
                    t = Vec::new();
                }

                blocks.push(block);
                index += consumed;
            }
            // Didn't find a block, assume it's a Paragraph
            None => {
                // Empty linebreak; push a new Paragraph
                if lines[index].is_empty() && !t.is_empty() {
                    blocks.push(Paragraph(t));
                    t = Vec::new();
                }

                // Parse any span elements in this line
                let spans = parse_spans(lines[index]);

                // Add a newline between linebreaks, unless there
                // is nothing
                match (t.last(), spans.first()) {
                    (_, None) => {}
                    (None, _) => {}
                    _ => t.push(Text("\n".to_owned())),
                }

                t.extend_from_slice(&spans);
                index += 1;
            }
        }
    }

    if !t.is_empty() {
        blocks.push(Paragraph(t));
    }

    blocks
}

fn parse_block(lines: &[&str]) -> Option<(Block, usize)> {
    pipe_opt!(
        lines
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
