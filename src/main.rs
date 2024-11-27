mod action;
mod colors;
mod editor;
mod mode;
mod buffer;
use anyhow::Ok;
use buffer::Buffer;
mod viewport;

fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().nth(1);
    let buffer = Buffer::new(file_path);

    let mut editor = editor::Editor::new(buffer)?;
    editor.enter_raw_mode()?;
    editor.run()?;
    drop(editor);
    
    Ok(())
}
