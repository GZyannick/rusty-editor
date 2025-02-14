use crate::{
    editor::{core::mode::Mode, Editor, TERMINAL_SIZE_MINUS},
    theme::colors,
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
            .on(Color::from(colors::FADED_PURPLE)),
    ))?;

    //print the filename
    editor.stdout.queue(PrintStyledContent(
        filename
            .with(Color::White)
            .on(Color::from(colors::DARK0_SOFT)),
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
            format!("{symbol}{cmd:<width$}", width = r_width - 1).on(Color::from(colors::DARK0)),
        ))?;
    Ok(())
}

pub fn draw_line_counter<W: Write>(editor: &mut Editor<W>, pos: String) -> Result<()> {
    // print the cursor position
    editor.stdout.queue(PrintStyledContent(
        pos.with(Color::Black).on(Color::from(colors::BRIGHT_GREEN)),
    ))?;

    Ok(())
}

#[cfg(test)]
mod test_draw_bottom {

    use super::*;
    use std::io::Cursor;

    // Helper function to mock Stdout with a Vec<u8>
    #[test]
    fn test_draw_bottom() {
        let mut editor = Editor::<Cursor<Vec<u8>>>::default();

        let result = draw_bottom(&mut editor);

        let output = editor.stdout.get_ref().clone();
        println!("output: {:#?}", output);
        assert!(
            result.is_ok(),
            "draw_bottom devrait s'exécuter sans erreurs"
        );
    }
}
