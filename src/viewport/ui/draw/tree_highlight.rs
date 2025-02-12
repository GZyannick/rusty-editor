use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

use crate::{theme::color_highligther::ColorHighligter, viewport::Viewport};

// highlight the rust code with tree_sitter and tree_sitter_rust
pub fn highlight(viewport: &Viewport, code: &String) -> anyhow::Result<Vec<ColorHighligter>> {
    let mut colors: Vec<ColorHighligter> = vec![];
    let mut parser = Parser::new();
    parser.set_language(&viewport.language)?;
    let tree = parser.parse(code, None).expect("tree_sitter couldnt parse");
    let mut query_cursor = QueryCursor::new();
    let mut query_matches =
        query_cursor.matches(&viewport.query, tree.root_node(), code.as_bytes());
    while let Some(m) = query_matches.next() {
        for cap in m.captures {
            let node = cap.node;
            let punctuation = viewport.query.capture_names()[cap.index as usize];

            colors.push(ColorHighligter::new_from_capture(
                node.start_byte(),
                node.end_byte(),
                punctuation,
            ))
        }
    }
    Ok(colors)
}
