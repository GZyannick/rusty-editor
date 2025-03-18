use crate::{
    editor::{core::mode::Mode, Editor, TERMINAL_SIZE_MINUS},
    THEME,
};
use anyhow::Result;
use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use std::io::Write;
pub fn draw_bottom<W: Write>(editor: &mut Editor<W>) -> anyhow::Result<()> {
    editor
        .stdout
        .queue(cursor::MoveTo(0, editor.size.1 - TERMINAL_SIZE_MINUS))?;

    let c_viewport = editor.viewports.c_viewport();
    let cursor_viewport = c_viewport.viewport_cursor(&editor.cursor);

    let mode = format!(" {} ", editor.mode);
    let pos = format!(" {}:{} ", cursor_viewport.0, cursor_viewport.1);
    let pad_width = editor.size.0 - mode.len() as u16 - pos.len() as u16 - TERMINAL_SIZE_MINUS;

    let filename = format!(
        " {:<width$} ",
        c_viewport.buffer.path,
        width = pad_width as usize
    );

    draw_status_line(editor, mode, filename)?;
    draw_line_counter(editor, pos)?;
    draw_last_line(editor)?;

    Ok(())
}
pub fn draw_status_line<W: Write>(
    editor: &mut Editor<W>,
    mode: String,
    filename: String,
) -> Result<()> {
    editor.stdout.queue(PrintStyledContent(
        mode.with(Color::White)
            .bold()
            .on(Color::from(THEME.faded_purple)),
    ))?;

    //print the filename
    editor.stdout.queue(PrintStyledContent(
        filename
            .with(Color::from(THEME.default))
            .on(Color::from(THEME.bg1)),
    ))?;
    Ok(())
}
// this method will draw command or search depending on the mode
pub fn draw_last_line<W: Write>(editor: &mut Editor<W>) -> Result<()> {
    let (symbol, cmd) = match editor.mode {
        Mode::Command => (':', &editor.command),
        Mode::Search => ('/', &editor.search),
        _ => (' ', &editor.command), // will print &self.command but will be empty, like that i
                                     // dont need to make String::new()
    };
    let r_width = editor.size.0 as usize - cmd.len();
    editor
        .stdout
        .queue(cursor::MoveTo(0, editor.size.1 - 1))?
        .queue(PrintStyledContent(
            format!("{symbol}{cmd:<width$}", width = r_width - 1).on(Color::from(THEME.bg0)),
        ))?;
    Ok(())
}

pub fn draw_line_counter<W: Write>(editor: &mut Editor<W>, pos: String) -> Result<()> {
    // print the cursor position
    editor.stdout.queue(PrintStyledContent(
        pos.with(Color::Black).on(Color::from(THEME.bright_green)),
    ))?;

    Ok(())
}

#[cfg(test)]
mod test_draw_bottom {
    use super::*;
    use std::io::Cursor;

    // Helper function to create a mock Editor with Cursor<Vec<u8>>
    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::<Cursor<Vec<u8>>>::default()
    }

    #[test]
    fn test_draw_bottom_success() {
        let mut editor = create_mock_editor();

        // Call the draw_bottom function
        let result = draw_bottom(&mut editor);

        // Check if the result is Ok
        assert!(result.is_ok(), "draw_bottom should execute without errors");

        // Extract the output from stdout (a Cursor<Vec<u8>>)
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        // Check if some expected content (like the mode or position) is printed
        assert!(
            output_str.contains(" "),
            "Output should contain space indicating status bar content."
        );
    }

    #[test]
    fn test_draw_status_line_success() {
        let mut editor = create_mock_editor();

        // Prepare some test strings for mode and filename
        let mode = "Normal ";
        let filename = "test_file.txt";

        // Call the draw_status_line function
        let result = draw_status_line(&mut editor, mode.to_string(), filename.to_string());

        // Check if the result is Ok
        assert!(
            result.is_ok(),
            "draw_status_line should execute without errors"
        );

        // Extract the output and check if it contains expected parts
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        assert!(output_str.contains(mode), "Output should contain the mode");
        assert!(
            output_str.contains(filename),
            "Output should contain the filename"
        );
    }

    #[test]
    fn test_draw_last_line_command_mode() {
        let mut editor = create_mock_editor();
        editor.mode = Mode::Command;
        editor.command = ":w".to_string(); // Simulate a command

        // Call the draw_last_line function
        let result = draw_last_line(&mut editor);

        // Check if the result is Ok
        assert!(
            result.is_ok(),
            "draw_last_line should execute without errors in Command mode"
        );

        // Extract the output and check if it contains the command
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        assert!(
            output_str.contains(":w"),
            "Output should contain the command"
        );
    }

    #[test]
    fn test_draw_last_line_search_mode() {
        let mut editor = create_mock_editor();
        editor.mode = Mode::Search;
        editor.search = "/search_term".to_string(); // Simulate a search term

        // Call the draw_last_line function
        let result = draw_last_line(&mut editor);

        // Check if the result is Ok
        assert!(
            result.is_ok(),
            "draw_last_line should execute without errors in Search mode"
        );

        // Extract the output and check if it contains the search term
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        assert!(
            output_str.contains("/search_term"),
            "Output should contain the search term"
        );
    }

    #[test]
    fn test_draw_line_counter() {
        let mut editor = create_mock_editor();
        let position = "10:20"; // Simulate cursor position

        // Call the draw_line_counter function
        let result = draw_line_counter(&mut editor, position.to_string());

        // Check if the result is Ok
        assert!(
            result.is_ok(),
            "draw_line_counter should execute without errors"
        );

        // Extract the output and check if it contains the position
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        assert!(
            output_str.contains(position),
            "Output should contain the position"
        );
    }
}
