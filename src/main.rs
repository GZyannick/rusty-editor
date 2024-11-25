mod editor;
mod mode;
mod action;

fn main() -> anyhow::Result<()>{
    let mut editor = editor::Editor::new()?;
    editor.enter_raw_mode()?;
    editor.draw()?;
    drop(editor);
    Ok(())
}
