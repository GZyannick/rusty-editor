mod action;
mod colors;
mod editor;
mod mode;
mod buffer;
use buffer::Buffer;




fn main() -> anyhow::Result<()> {
    let file_path = std::env::args().nth(1);
    let mut buffer = Buffer::new(file_path);

    //buffer.add_char('h', &(0, 0));

    let mut editor = editor::Editor::new(buffer)?;
    editor.enter_raw_mode()?;
    editor.run()?;
    drop(editor);
    
    Ok(())
}
