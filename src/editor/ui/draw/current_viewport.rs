use crate::editor::Editor;
pub fn draw_current_viewport(editor: &mut Editor) -> anyhow::Result<()> {
    let current_viewport = editor.viewports.c_viewport();
    {
        match editor.is_visual_mode() {
            true => {
                // give us two option of (u16, u16) first is start second is end
                if let Some(v_block) = editor.get_visual_block_pos() {
                    current_viewport.draw(
                        &mut editor.stdout,
                        Some(v_block.start),
                        Some(v_block.end),
                    )?;
                };
            }
            false => {
                // draw normal viewport without visual block
                current_viewport.draw(&mut editor.stdout, None, None)?;
            }
        }
    }
    Ok(())
}
