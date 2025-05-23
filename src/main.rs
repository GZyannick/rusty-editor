use std::io::stdout;
use std::panic;
use std::sync::{Arc, Mutex, OnceLock};
mod buff;
mod languages;
mod theme;
mod viewports;
use buff::Buffer;
mod helper;
use crossterm::{terminal, ExecutableCommand};
use helper::logger::Logger;

mod editor;
use editor::Editor;
pub const LINE_NUMBERS_WIDTH: u16 = 5;
pub static INSTANCE: OnceLock<Mutex<Logger>> = OnceLock::new();

use anyhow::Ok;
use once_cell::sync::Lazy;
use theme::Theme;
mod viewport;

pub static THEME: Lazy<Arc<Theme>> = Lazy::new(|| Arc::new(Theme::load_theme().unwrap()));
fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().nth(1);
    let buffer = Buffer::new(file_path.clone());
    let mut editor = Editor::new(buffer, stdout())?;
    editor.enter_raw_mode()?;

    panic::set_hook(Box::new(|info| {
        let _ = stdout().execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
        eprintln!("{}", info)
    }));

    editor.run()?;

    Ok(())
}

#[cfg(test)]
mod main_tests {
    use std::io::stdout;

    use crate::{
        buff::Buffer,
        editor::{core::mode::Mode, Editor},
    };

    #[test]
    fn check_file_path_buffer() {
        let path = "./src/main.rs".to_string();
        let file_path = Some(path.clone());
        let buffer = Buffer::new(file_path);
        assert!(!buffer.is_directory);
        assert_eq!(buffer.path, path);
    }
    #[test]
    fn check_folder_path_buffer() {
        let file_path = Some("./".to_string());
        let buffer = Buffer::new(file_path);
        assert!(buffer.is_directory);
        assert_eq!(buffer.path, "./".to_string());
    }

    #[test]
    fn check_empty_path_buffer() {
        let file_path = None;
        let buffer = Buffer::new(file_path);
        assert!(!buffer.is_directory);
        assert_eq!(buffer.path, "Empty".to_string());
    }

    #[test]
    fn check_editor() -> anyhow::Result<()> {
        let path = "./src/main.rs".to_string();
        let file_path = Some(path.clone());
        let buffer = Buffer::new(file_path);
        let editor = Editor::new(buffer, stdout())?;
        assert!(matches!(editor.mode, Mode::Normal));
        assert!(editor.viewports.values.len() == 1);
        Ok(())
    }
}
