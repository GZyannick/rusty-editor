use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::{theme::colors, viewport::Viewport};

use super::{tree_highlight::highlight, visual_block::draw_block};

pub fn draw_file<W: std::io::Write>(
    viewport: &Viewport,
    stdout: &mut W,
    start_v_mode: Option<(u16, u16)>,
    end_v_mode: Option<(u16, u16)>,
) -> anyhow::Result<u16> {
    let v_width = viewport.vwidth;
    let viewport_buffer = viewport.viewport();
    let colors = highlight(viewport, &viewport_buffer)?;

    let mut y: u16 = viewport.min_vheight;
    let mut x: u16 = 0;

    let mut colorhighligter = None;

    // let chars_len = viewport_buffer.len().wrapping_sub(1);
    let chars_len = viewport_buffer.len().saturating_sub(1);
    let mut bg_color = viewport.bg_color;

    for (pos, c) in viewport_buffer.chars().enumerate() {
        // tell us that we are at the end of the line
        // so we draw the line number and empty char to end of terminal size to get the same bg
        // and dont have undesirable artifact like ghost char
        if c == '\n' {
            viewport.draw_line_number(stdout, y)?;
            stdout.queue(cursor::MoveTo(x + viewport.min_vwidth, y))?;
            if x < viewport.vwidth {
                stdout.queue(PrintStyledContent(
                    " ".repeat(v_width as usize - x as usize)
                        .on(viewport.bg_color),
                ))?;
            }
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
                    y,
                    start_block,
                    end_block,
                    Color::from(colors::LIGTH_GREY),
                );
            }
        }

        // if we are in search mode and we found occurences draw them
        if !viewport.search_pos.is_empty() {
            if let Some(color) = viewport.draw_search(x, y) {
                bg_color = color
            }
        }

        // change char color if its highlight
        let styled_char = match colorhighligter {
            Some(ch) => format!("{c}").on(bg_color).with(ch.color),
            None => format!("{c}",).on(bg_color),
        };

        // move cursor to draw the char
        stdout
            .queue(cursor::MoveTo(x + viewport.min_vwidth, y))?
            .queue(PrintStyledContent(styled_char))?;

        x += 1;

        // if we are at the end of the string
        if pos == chars_len {
            viewport.draw_line_number(stdout, y)?;
            stdout.queue(cursor::MoveTo(x + viewport.min_vwidth, y))?;
            if x < viewport.vwidth {
                stdout.queue(PrintStyledContent(
                    " ".repeat(v_width as usize - x as usize)
                        .on(viewport.bg_color),
                ))?;
            }
            y += 1
        }
    }
    Ok(y)
}

#[cfg(test)]
mod tests_draw_file {
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
        };

        let viewport = Viewport {
            buffer,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&viewport, &mut mock_stdout, None, None);

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
        };

        let viewport = Viewport {
            buffer,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&viewport, &mut mock_stdout, None, None);

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
        };

        let viewport = Viewport {
            buffer,
            search_pos: vec![(5, 5, 3)], // Search result at "main"
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = draw_file(&viewport, &mut mock_stdout, None, None);

        assert!(
            result.is_ok(),
            "draw_file() should succeed with search results"
        );
    }
}
