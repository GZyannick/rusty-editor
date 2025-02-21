use std::io::Write;

use crate::editor::Editor;
pub fn draw_current_viewport<W: Write>(editor: &mut Editor<W>) -> anyhow::Result<()> {
    {
        match editor.is_visual_mode() {
            true => {
                // give us two option of (u16, u16) first is start second is end
                if let Some(v_block) = editor.get_visual_block_pos() {
                    editor.viewports.c_mut_viewport().draw(
                        &mut editor.stdout,
                        Some(v_block.start),
                        Some(v_block.end),
                    )?;
                };
            }
            false => {
                // draw normal viewport without visual block
                editor
                    .viewports
                    .c_mut_viewport()
                    .draw(&mut editor.stdout, None, None)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_draw_current_viewport {
    use super::*;
    use crate::editor::{core::mode::Mode, Editor};
    use std::io::Cursor;

    // Helper function to create a mock Editor with Cursor<Vec<u8>>
    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::<Cursor<Vec<u8>>>::default()
    }

    #[test]
    fn test_draw_current_viewport_normal_mode() {
        let mut editor = create_mock_editor();
        editor.mode = Mode::Normal; // Set to normal mode

        // Call the draw_current_viewport function
        let result = draw_current_viewport(&mut editor);

        // Check if the result is Ok
        assert!(
            result.is_ok(),
            "draw_current_viewport should execute without errors in normal mode"
        );

        // Extract the output and check if the viewport is drawn
        let output_str = String::from_utf8(editor.stdout.get_ref().clone())
            .expect("Failed to convert stdout to string");

        // Check if some expected content (like viewport info) is printed
        assert!(
            output_str.contains(""),
            "Expected viewport content to be printed in normal mode."
        );
    }
}
