use anyhow::Result;
use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::theme::colors;

#[derive(Debug)]
pub struct Popup {
    pub top: u16,
    pub left: u16,
    pub width: u16,
    pub height: u16,
}

const POPUP_PERCENTAGE: u16 = 30;
impl Popup {
    fn percentage_of(n: u16) -> u16 {
        (n * POPUP_PERCENTAGE) / 100
    }
    fn wrapping_sub_by_percentage(n: u16) -> u16 {
        n.wrapping_sub(Popup::percentage_of(n))
    }

    pub fn new(editor_size: &(u16, u16)) -> Result<Self> {
        let left = Popup::percentage_of(editor_size.0) / 2;
        let top = Popup::percentage_of(editor_size.1) / 2;
        let width = Popup::wrapping_sub_by_percentage(editor_size.0);
        let height = Popup::wrapping_sub_by_percentage(editor_size.1);

        Ok(Popup {
            top,
            left,
            width,
            height,
        })
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout) -> Result<()> {
        for x in self.left..(self.width + self.left) {
            for y in self.top..(self.height + self.top) {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(PrintStyledContent(" ".on(Color::from(colors::DARK2))))?;
            }
        }

        Ok(())
    }

    // TODO: Not used for now
    pub fn _resize(&mut self, editor_size: &(u16, u16)) {
        self.width = Popup::percentage_of(editor_size.0);
        self.height = Popup::percentage_of(editor_size.1);
    }
}
