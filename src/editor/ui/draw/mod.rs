use crate::editor::Editor;
use anyhow::Result;
use crossterm::{cursor, QueueableCommand};
use std::io::Write;

mod bottom;
mod current_viewport;
mod modal;

impl<W: Write> Editor<W> {
    pub fn draw(&mut self) -> Result<()> {
        // some terminal line windows default show the cursor when drawing the tui so hide and show
        // it at the end of draw
        self.stdout.queue(cursor::Hide)?;

        current_viewport::draw_current_viewport(self)?;
        modal::draw_modal(self)?;

        if !self.toast.is_empty() {
            self.toast.draw(&mut self.stdout, &self.size.0)?;
        }

        bottom::draw_bottom(self)?;

        let c_viewport = self.viewports.c_viewport();
        self.stdout.queue(cursor::MoveTo(
            self.cursor.0 + c_viewport.min_vwidth,
            self.cursor.1 + c_viewport.min_vheight,
        ))?;

        self.stdout.queue(cursor::Show)?;
        self.stdout.flush()?;
        Ok(())
    }
}
