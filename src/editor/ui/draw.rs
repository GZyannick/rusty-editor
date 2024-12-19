use std::io::Write;

use crate::{
    editor::{Editor, TERMINAL_SIZE_MINUS},
    theme::colors,
    viewport::Viewport,
};
use anyhow::Result;
use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

impl Editor {
    pub fn draw(&mut self) -> Result<()> {
        // some terminal line windows default show the cursor when drawing the tui so hide and show
        // it at the end of draw
        self.stdout.queue(cursor::Hide)?;

        self.draw_current_viewport()?;
        self.draw_bottom()?;

        let c_viewport = self.c_viewport();
        self.stdout.queue(cursor::MoveTo(
            self.cursor.0 + c_viewport.min_vwidth,
            self.cursor.1 + c_viewport.min_vheight,
        ))?;

        self.stdout.queue(cursor::Show)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_current_viewport(&mut self) -> anyhow::Result<()> {
        // self.c_mut_viewport().draw(&mut self.stdout)?;
        Ok(())
    }

    fn draw_bottom(&mut self) -> anyhow::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - TERMINAL_SIZE_MINUS))?;

        let c_viewport = self.c_viewport();
        let cursor_viewport = c_viewport.viewport_cursor(&self.cursor);

        let mode = format!(" {} ", self.mode);
        let pos = format!(" {}:{} ", cursor_viewport.0, cursor_viewport.1);
        let pad_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - TERMINAL_SIZE_MINUS;
        let filename = format!(
            " {:<width$} ",
            c_viewport.buffer.path,
            width = pad_width as usize
        );

        self.draw_status_line(mode, filename)?;
        self.draw_line_counter(pos)?;
        self.draw_command_line()?;

        Ok(())
    }

    fn draw_status_line(&mut self, mode: String, filename: String) -> Result<()> {
        self.stdout.queue(PrintStyledContent(
            mode.with(Color::White)
                .bold()
                .on(Color::from(colors::FADED_PURPLE)),
        ))?;

        //print the filename
        self.stdout.queue(PrintStyledContent(
            filename
                .with(Color::White)
                .on(Color::from(colors::DARK0_SOFT)),
        ))?;
        Ok(())
    }

    fn draw_line_counter(&mut self, pos: String) -> Result<()> {
        // print the cursor position
        self.stdout.queue(PrintStyledContent(
            pos.with(Color::Black).on(Color::from(colors::BRIGHT_GREEN)),
        ))?;

        Ok(())
    }

    fn draw_command_line(&mut self) -> Result<()> {
        let cmd = &self.command;
        let r_width = self.size.0 as usize - cmd.len();
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - 1))?
            .queue(PrintStyledContent(
                format!(":{cmd:<width$}", width = r_width - 1).on(Color::from(colors::DARK0)),
            ))?;
        Ok(())
    }
}
