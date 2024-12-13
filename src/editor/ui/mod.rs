// i use this trait to separate the draw part of the editor and the keyHandling Part
pub mod clear;
pub mod popup;
use anyhow::Result;
pub trait Draw {
    fn draw(&mut self) -> Result<()>;
    fn draw_bottom(&mut self) -> Result<()>;
    fn draw_status_line(&mut self, mode: String, filename: String) -> Result<()>;
    fn draw_line_counter(&mut self, pos: String) -> Result<()>;
    fn draw_command_line(&mut self) -> Result<()>;
}
