mod action;
mod colors;
mod editor;
mod mode;
mod buffer;
use anyhow::Ok;
use buffer::Buffer;

// to implement scrolling and showing text of the size of our current terminal
pub struct Viewport {
    pub buffer: Buffer,
    pub x_pos: u16, 
    pub y_pos: u16,
    pub width: u16,
    pub height: u16,
}

impl Viewport {
    pub fn new(buffer: Buffer, width: u16, height: u16) -> Viewport {
        Viewport {
            buffer,
            width: width - 2,
            height: height - 2,
            x_pos: 0,
            y_pos: 0,
        }
    }

   pub fn get_buffer_viewport(&mut self) -> &[String] {
        let start = self.y_pos as usize;
        let end = (self.x_pos + self.height) as usize;
        &self.buffer.lines[start..end]
   } 
}


fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().nth(1);
    let buffer = Buffer::new(file_path);

    let mut editor = editor::Editor::new(buffer)?;
    editor.enter_raw_mode()?;
    editor.run()?;
    drop(editor);
    
    Ok(())
}
