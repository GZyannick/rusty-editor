use crate::editor::Editor;

pub fn draw_modal(editor: &mut Editor) -> anyhow::Result<()> {
    if let Some(modal) = editor.modal.take() {
        modal.draw_modal(editor)?;
        editor.modal = Some(modal);
    }
    Ok(())
}
