use std::io::Write;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::{theme::icon, viewport::Viewport};

pub fn draw_file_explorer<W: Write>(viewport: &Viewport, stdout: &mut W) -> anyhow::Result<u16> {
    let mut y = viewport.min_vheight;
    let start = viewport.top as usize;
    let end = (viewport.top + viewport.max_vheight()) as usize;
    let range = match end > viewport.buffer.lines.len() {
        true => start..viewport.buffer.lines.len(),
        false => start..end,
    };
    for (i, line) in viewport.buffer.lines[range].iter().enumerate() {
        viewport.draw_line_number(stdout, y)?;

        let icon = icon::get_icon(line);
        // we skip the ../ line
        //            // we skip the ../ line
        let line = match i > 0 || (i == 0 && line != "../") {
            true if line.starts_with(viewport.buffer.path.as_str()) => {
                let mut path = viewport.buffer.path.clone();
                if !path.ends_with("/") {
                    path.push('/');
                }
                line.replacen(path.as_str(), "", 1).to_string()
            }
            true => line.to_string(),
            false => line.to_string(),
        };

        let path = format!(" {:<width$} ", line, width = viewport.vwidth as usize - 4);
        stdout
            .queue(cursor::MoveTo(viewport.min_vwidth - 1, y))?
            .queue(PrintStyledContent(
                icon.with(Color::White).on(viewport.bg_color).bold(),
            ))?
            .queue(cursor::MoveTo(viewport.min_vwidth + 1, y))?
            .queue(PrintStyledContent(
                path.with(Color::White).on(viewport.bg_color),
            ))?;
        y += 1;
    }
    Ok(y)
}

#[cfg(test)]
mod test_draw_file_explorer {
    use crate::buff::Buffer;

    use super::*;
    use crossterm::style::Color;
    use std::{io::Cursor, path::PathBuf};

    // Function to create a viewport with mock files
    fn create_viewport_with_files() -> Viewport {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR")); // Récupère la racine du projet
        path.push("tests_data/test_file_explorer_folder");
        Viewport {
            buffer: Buffer::new(Some(path.display().to_string())),
            vwidth: 40,
            min_vwidth: 5,
            bg_color: Color::Black,
            ..Viewport::default()
        }
    }

    fn create_mock_stdout() -> Cursor<Vec<u8>> {
        Cursor::new(Vec::new()) // Create a new Cursor to capture the output
    }

    #[test]
    fn test_draw_file_explorer() {
        let viewport = create_viewport_with_files();
        let mut mock_stdout = create_mock_stdout();
        let result = draw_file_explorer(&viewport, &mut mock_stdout);

        assert!(
            result.is_ok(),
            "draw_file_explorer() should not return an error"
        );

        // Capture the output from the Cursor
        let output = String::from_utf8(mock_stdout.into_inner()).unwrap();

        // check directory
        assert!(
            output.contains("\u{f115}"),
            "output should contain directory icon"
        );
        assert!(
            output.contains("user"),
            "output should contain directory icon"
        );

        assert!(
            output.contains("\u{f15c}"),
            "output should contain txt icon"
        );

        assert!(
            output.contains("file1.txt"),
            "output should contain txt file"
        );

        // // check file2.rs
        assert!(output.contains("\u{e7a8}"), "output should contain rs icon");
        assert!(output.contains("file2.rs"), "output should contain rs file");

        // // check file3.md
        assert!(output.contains("\u{f48a}"), "output should contain md icon");
        assert!(output.contains("file3.md"), "output should contain md file");

        // // check image.png
        assert!(
            output.contains("\u{f1c5}"),
            "output should contain png icon"
        );
        assert!(
            output.contains("image.png"),
            "output should contain image file"
        );
    }

    #[test]
    fn test_draw_file_explorer_with_no_files() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests_data/test_file_explorer_folder/user");

        let viewport = Viewport {
            buffer: Buffer::new(Some(path.display().to_string())),
            vwidth: 40,
            min_vwidth: 5,
            bg_color: Color::Black,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();

        let result = draw_file_explorer(&viewport, &mut mock_stdout);

        assert!(
            result.is_ok(),
            "draw_file_explorer() should not return an error with no files"
        );

        // check that no output was produced
        let output = String::from_utf8(mock_stdout.into_inner()).unwrap();
        assert!(output.contains("../"), "output should be contains ../");
    }
}
