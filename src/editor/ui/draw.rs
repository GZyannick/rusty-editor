use std::io::Write;

use crate::{
    editor::{core::mode::Mode, modal_input::ModalContent, Editor, TERMINAL_SIZE_MINUS},
    theme::colors,
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
        if let Some(modal) = self.modal.take() {
            self.draw_modal(&*modal)?;
            self.modal = Some(modal);
        }
        if !self.toast.is_empty() {
            self.toast.draw(&mut self.stdout, &self.size.0)?;
        }
        self.draw_bottom()?;

        let c_viewport = self.viewports.c_viewport();
        self.stdout.queue(cursor::MoveTo(
            self.cursor.0 + c_viewport.min_vwidth,
            self.cursor.1 + c_viewport.min_vheight,
        ))?;

        self.stdout.queue(cursor::Show)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_current_viewport(&mut self) -> anyhow::Result<()> {
        let current_viewport = self.viewports.c_viewport();
        {
            match self.is_visual_mode() {
                true => {
                    // give us two option of (u16, u16) first is start second is end
                    if let Some(v_block) = self.get_visual_block_pos() {
                        current_viewport.draw(
                            &mut self.stdout,
                            Some(v_block.start),
                            Some(v_block.end),
                        )?;
                    };
                }
                false => {
                    current_viewport.draw(&mut self.stdout, None, None)?;
                }
            }
        }
        Ok(())
    }

    fn draw_bottom(&mut self) -> anyhow::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - TERMINAL_SIZE_MINUS))?;

        let c_viewport = self.viewports.c_viewport();
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
        self.draw_last_line()?;

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

    // this method will draw command or search depending on the mode
    fn draw_last_line(&mut self) -> Result<()> {
        let (symbol, cmd) = match self.mode {
            Mode::Command => (':', &self.command),
            Mode::Search => ('/', &self.search),
            _ => (' ', &self.command), // will print &self.command but will be empty, like that i
                                       // dont need to make String::new()
        };
        let r_width = self.size.0 as usize - cmd.len();
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - 1))?
            .queue(PrintStyledContent(
                format!("{symbol}{cmd:<width$}", width = r_width - 1)
                    .on(Color::from(colors::DARK0)),
            ))?;
        Ok(())
    }

    fn draw_modal(&mut self, modal: &dyn ModalContent) -> Result<()> {
        let width = self.size.0;
        let height = self.size.1;
        let modal_width = width / 4;
        let modal_height = height / 4;

        let start_x = (width - modal_width) / 2;
        let start_y = (height - modal_height) / 2;

        let title = format!(" {:<width$}", modal.title(), width = modal_width as usize);
        self.stdout.queue(cursor::MoveTo(start_x, start_y))?;
        self.stdout.queue(PrintStyledContent(
            title.bold().on(Color::from(colors::FADED_PURPLE)),
        ))?;

        let body = format!(" {:<width$}", modal.body(), width = modal_width as usize);
        self.stdout.queue(cursor::MoveTo(start_x, start_y + 1))?;
        self.stdout
            .queue(PrintStyledContent(body.on(Color::from(colors::DARK0_SOFT))))?;

        self.stdout.flush()?;
        Ok(())
    }
}
