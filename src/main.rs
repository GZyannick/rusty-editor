use std::io::Read;
use std::fs::File;

mod editor;
mod mode;
mod action;
mod colors;


#[derive(Debug)]
pub struct Buffer {
    file: Option<File>,
    lines: Vec<String>,
}

impl Buffer {
    pub fn new(file_path: Option<String>) -> anyhow::Result<Buffer> {
        let mut file = None;
        let mut lines: Vec<String> = Vec::new();

        if let Some(path) = file_path {
            match File::open(path) {
                Ok(mut c_file) => {
                    let mut buf = String::new();
                    c_file.read_to_string(&mut buf)?;
                    file = Some(c_file);
                    lines = buf.lines().map(|s| s.to_string()).collect()
                }
                Err(err) => {
                    return Err(err.into());
                },
            }
        }
        Ok(Buffer {
            file,
            lines,
        })
    }
}

fn main() -> anyhow::Result<()>{
    let file_path = std::env::args().nth(1);
    let buffer = Buffer::new(file_path)?;



    let mut editor = editor::Editor::new(buffer)?;
    editor.enter_raw_mode()?;
    editor.run()?;
    drop(editor);
    Ok(())
}
