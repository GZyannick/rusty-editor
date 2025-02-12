use std::io::stdout;
use std::panic;
use std::sync::{Mutex, OnceLock};
mod buff;
mod theme;
mod viewports;
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
    let buffer = Buffer::new(file_path.clone());
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

#[cfg(test)]
mod main_tests {
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

        let editor = Editor::new(buffer)?;
        assert!(matches!(editor.mode, Mode::Normal));
        assert!(!editor.keybinds.normal_mode.is_empty());
        assert!(!editor.keybinds.visual_mode.is_empty());
        assert!(!editor.keybinds.insert_mode.is_empty());
        assert!(!editor.keybinds.command_mode.is_empty());
        assert!(!editor.keybinds.file_explorer.is_empty());
        assert!(!editor.keybinds.search_mode.is_empty());
        assert!(editor.viewports.values.len() == 2);
        Ok(())
    }
}
