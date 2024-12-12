use std::io::stdout;
use std::panic;
use std::sync::{Mutex, OnceLock};
mod theme;

mod buff;
use buff::Buffer;

mod helper;
use crossterm::{terminal, ExecutableCommand};
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

    panic::set_hook(Box::new(|info| {
        let _ = stdout().execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        eprintln!("{}", info)
    }));

    editor.run()?;

    Ok(())
}
