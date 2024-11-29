use std::sync::{OnceLock, Mutex};

mod theme;

mod buff;
use buff::Buffer;

mod helper;
use helper::logger::Logger;

mod editor;
use editor::Editor;

pub static INSTANCE: OnceLock<Mutex<Logger>> = OnceLock::new();

use anyhow::Ok;
mod viewport;

fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().nth(1);
    let buffer = Buffer::new(file_path);
    let mut editor = Editor::new(buffer)?;
    editor.enter_raw_mode()?;
    editor.run()?;
    Ok(())
}
