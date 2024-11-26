use std::fs::{self, File};
use std::io::Read;

mod action;
mod colors;
mod editor;
mod mode;

#[derive(Debug)]
pub struct Buffer {
    pub file: Option<File>,
    pub path: String,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new(file_path: Option<String>) -> Buffer {
        if let Some(f_path) = file_path {
            match fs::metadata(f_path.clone()) {
                Ok(metadata) if metadata.is_file() => {
                    return Buffer::from_file(&f_path);
                }
                Ok(metadata) if metadata.is_dir() => {
                    return Buffer::from_dir(&f_path);
                }
                _ => (),
            }
        }

        Buffer { file: None, lines: vec![], path: "Empty".to_string() }
    }

    fn from_file(f_path: &str) -> Buffer {
        let mut file = None;
        let mut lines: Vec<String> = Vec::new();
        let mut path = String::from("Empty");

        if let Ok(mut c_file) = File::open(f_path) {
            let mut buf = String::new();
            c_file.read_to_string(&mut buf).unwrap();
            file = Some(c_file);
            lines = buf.lines().map(|s| s.to_string()).collect();
            path = f_path.to_string();
        }

        Buffer {
            file,
            lines,
            path,
        }
    }

    fn from_dir(_f_path: &str) -> Buffer {
        //TODO Handle dir path for now it will return an empty buffer
        Buffer { file: None, lines: vec![], path: "Empty".to_string() }
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
