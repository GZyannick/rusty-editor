mod editor;
mod mode;
mod action;
mod colors;

fn main() -> anyhow::Result<()>{
    let mut editor = editor::Editor::new()?;
    editor.enter_raw_mode()?;
    editor.run()?;
    drop(editor);
    Ok(())
}
