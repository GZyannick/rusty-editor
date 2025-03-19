use std::io::BufWriter;

use anyhow::Result;
use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use super::{tree_highlight::highlight, visual_block::draw_block};
use crate::{viewport::Viewport, THEME};

fn draw_new_line<W: std::io::Write>(
    viewport: &Viewport,
    stdout: &mut W,
    x: &mut u16,
    y: &mut u16,
) -> Result<()> {
    viewport.draw_line_number(stdout, *y)?;
    stdout.queue(cursor::MoveTo(*x + viewport.min_vwidth, *y))?;
    if *x < viewport.vwidth {
        stdout.queue(PrintStyledContent(
            " ".repeat(viewport.vwidth as usize - *x as usize)
                .on(viewport.bg_color),
        ))?;
    }
    Ok(())
}

pub fn draw_file<W: std::io::Write>(
    viewport: &mut Viewport,
    stdout: &mut W,
    start_v_mode: Option<(u16, u16)>,
    end_v_mode: Option<(u16, u16)>,
) -> anyhow::Result<u16> {
    let viewport_buffer = viewport.viewport();
    let mut buffer = BufWriter::new(Vec::new());

    let colors = match viewport.last_highlighted_code != viewport_buffer {
        true => {
            let highlight = highlight(viewport, &viewport_buffer)?;
            viewport.cached_highlight = Some(highlight.clone());
            viewport.last_highlighted_code = viewport_buffer.clone();
            highlight
        }
        false => viewport.cached_highlight.clone().unwrap_or_default(),
    };

    let mut y: u16 = viewport.min_vheight;
    let mut x: u16 = 0;

    let mut colorhighligter = None;

    let chars_len = viewport_buffer.len().saturating_sub(1);
    let mut bg_color = viewport.bg_color;

    for (pos, c) in viewport_buffer.char_indices() {
        // tell us that we are at the end of the line
        // so we draw the line number and empty char to end of terminal size to get the same bg
        // and dont have undesirable artifact like ghost char
        if c == '\n' {
            draw_new_line(viewport, &mut buffer, &mut x, &mut y)?;
            x = 0;
            y += 1;
            continue;
        }

        // let us know if the current char is part of an highlight
        if let Some(colorh) = colors.iter().find(|ch| pos == ch.start) {
            colorhighligter = Some(colorh);
        } else if colors.iter().any(|ch| pos == ch.end) {
            colorhighligter = None
        }

        // allow us to change the bg_color to draw the visual_block
        if let Some(start_block) = start_v_mode {
            if let Some(end_block) = end_v_mode {
                bg_color = draw_block(
                    viewport,
                    x,
                    y.saturating_sub(viewport.min_vheight),
                    start_block,
                    end_block,
                    Color::from(THEME.light_gray),
                );
            }
        }

        // if we are in search mode and we found occurences draw them
        if !viewport.search_pos.is_empty() {
            if let Some(color) = viewport.draw_search(x, y) {
                bg_color = color
            }
        }

        let styled_char = match colorhighligter {
            Some(ch) => c.on(bg_color).with(ch.color),
            None => c.on(bg_color),
        };

        // move cursor to draw the char
        buffer
            .queue(cursor::MoveTo(x + viewport.min_vwidth, y))?
            .queue(PrintStyledContent(styled_char))?;
        // stdout
        //     .queue(cursor::MoveTo(x + viewport.min_vwidth, y))?
        //     .queue(PrintStyledContent(styled_char))?;

        x += 1;

        // if we are at the end of the string
        if pos == chars_len {
            draw_new_line(viewport, &mut buffer, &mut x, &mut y)?;
            y += 1
        }
    }
    stdout.write_all(&buffer.into_inner()?)?;
    stdout.flush()?;
    Ok(y)
}

#[cfg(test)]
mod tests_draw_file {
    use tree_sitter::Query;

    use crate::buff::Buffer;

    use super::*;
    use std::io::Cursor;

    fn create_mock_stdout() -> Cursor<Vec<u8>> {
        Cursor::new(Vec::new()) // Create a new Cursor to capture the output
    }

    // Test with an empty buffer
    #[test]
    fn test_draw_file_empty_buffer() {
        let buffer = Buffer {
            file: None,
            is_directory: false,
            path: "".to_string(),
            lines: vec![], // Empty buffer
            query_language: None,
        };

        let mut viewport = Viewport {
            buffer,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&mut viewport, &mut mock_stdout, None, None);

        assert!(
            result.is_ok(),
            "draw_file() should succeed even with an empty buffer"
        );
    }

    // Test with file content
    #[test]
    fn test_draw_file_with_content() {
        let buffer = Buffer {
            file: None,
            is_directory: false,
            path: "example.rs".to_string(),
            lines: vec![
                "fn main() {".to_string(),
                "    let x = 42;".to_string(),
                "    println!(\"{{}}\", x);".to_string(),
                "}".to_string(),
            ],
            query_language: Some((
                Query::new(
                    &tree_sitter_rust::LANGUAGE.into(),
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                )
                .expect("QueryErr"),
                tree_sitter_rust::LANGUAGE.into(),
            )),
        };

        let mut viewport = Viewport {
            buffer,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&mut viewport, &mut mock_stdout, None, None);

        assert!(
            result.is_ok(),
            "draw_file() should succeed with content in buffer"
        );
    }

    // Test with search enabled
    #[test]
    fn test_draw_file_with_search() {
        let buffer = crate::buff::Buffer {
            file: None,
            is_directory: false,
            path: "example.rs".to_string(),
            lines: vec![
                "fn main() {".to_string(),
                "    let x = 42;".to_string(),
                "    println!(\"{{}}\", x);".to_string(),
                "}".to_string(),
            ],
            query_language: Some((
                Query::new(
                    &tree_sitter_rust::LANGUAGE.into(),
                    tree_sitter_rust::HIGHLIGHTS_QUERY,
                )
                .expect("QueryErr"),
                tree_sitter_rust::LANGUAGE.into(),
            )),
        };

        let mut viewport = Viewport {
            buffer,
            search_pos: vec![(5, 5, 3)], // Search result at "main"
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&mut viewport, &mut mock_stdout, None, None);

        assert!(
            result.is_ok(),
            "draw_file() should succeed with search results"
        );
    }
}
