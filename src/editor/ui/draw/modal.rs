use std::io::Write;

use crate::editor::Editor;

pub fn draw_modal<W: Write>(editor: &mut Editor<W>) -> anyhow::Result<()> {
    if let Some(modal) = editor.modal.take() {
        modal.draw_modal(editor)?;
        editor.modal = Some(modal);
    }
    Ok(())
}
