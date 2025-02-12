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

pub fn draw_bottom(editor: &mut Editor) -> anyhow::Result<()> {
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
pub fn draw_status_line(editor: &mut Editor, mode: String, filename: String) -> Result<()> {
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
pub fn draw_last_line(editor: &mut Editor) -> Result<()> {
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

pub fn draw_line_counter(editor: &mut Editor, pos: String) -> Result<()> {
    // print the cursor position
    editor.stdout.queue(PrintStyledContent(
        pos.with(Color::Black).on(Color::from(colors::BRIGHT_GREEN)),
    ))?;

    Ok(())
}
