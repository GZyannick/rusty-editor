use std::io::Write;

use anyhow::Ok;

use crate::{
    editor::{core::mode::Mode, Editor},
    helper::clipboard::copy_to_clipboard,
};

use super::action::{Action, OldCursorPosition};

impl Action {
    pub fn deletion<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::RemoveCharAt => {
                // remove char at doenst take parameter because if we need to change at a specific
                // position we set the cursor before like that it simplified how the keybind work
                // in handle keybind
                let v_cursor = editor.v_cursor();
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
                let top = editor.viewports.c_viewport().top;
                if editor.viewports.c_viewport().get_line_len(&v_cursor) > 0 {
                    let char = editor
                        .viewports
                        .c_mut_viewport()
                        .buffer
                        .remove_char(v_cursor);

                    if let Some(char) = char {
                        let old_cursor = OldCursorPosition::new(editor.cursor, top);
                        editor
                            .undo_actions
                            .push(Action::UndoRemoveCharAt(old_cursor, char));
                    }
                }
            }

            Action::RemoveModalChar => {
                if let Some(ref mut modal) = editor.modal {
                    modal.pop();
                }
            }
            Action::RemoveChar => {
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
                let cursor_viewport = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();
                match cursor_viewport.0 > 0 {
                    true => {
                        editor.cursor.0 -= 1;
                        current_viewport
                            .buffer
                            .remove_char((cursor_viewport.0 - 1, cursor_viewport.1));
                    }
                    false if cursor_viewport.1 > 0 => {
                        // we get the size of the prev line before change
                        // because we want the text that will be added behind the cursor
                        let new_x_pos = current_viewport
                            .get_line_len_no_v_cursor(&(editor.cursor.0, editor.cursor.1 - 1));
                        current_viewport.buffer.remove_char_line(cursor_viewport);
                        editor.move_prev_line();
                        editor.cursor.0 = new_x_pos;
                    }
                    _ => {}
                }
            }
            Action::DeleteLine => {
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
                let (_, y) = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();
                let content = current_viewport.buffer.get(y as usize).clone();
                current_viewport.buffer.remove(y as usize);

                if let Some(text) = &content {
                    copy_to_clipboard(text);
                }

                editor.undo_actions.push(Action::UndoDeleteLine(
                    OldCursorPosition::new(editor.cursor, current_viewport.top),
                    content,
                ));
            }

            Action::DeleteWord => {
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
                let v_cursor = editor.v_cursor();
                editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .remove_word(v_cursor)
            }

            Action::RemoveCharFrom(is_search) => {
                let content = match is_search {
                    true => &mut editor.search,
                    false => &mut editor.command,
                };

                match content.is_empty() {
                    true => {
                        editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
                    }
                    false => {
                        content.pop();
                        if *is_search {
                            editor.buffer_actions.push(Action::FindSearchValue)
                        }
                    }
                }
            }

            Action::DeleteBlock => {
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
                if let Some(v_block) = editor.get_visual_block_pos() {
                    let c_mut_viewport = editor.viewports.c_mut_viewport();
                    let v_cursor_start = c_mut_viewport.viewport_cursor(&v_block.start);
                    let v_cursor_end = c_mut_viewport.viewport_cursor(&v_block.end);

                    let block_content: Vec<Option<String>> = c_mut_viewport
                        .buffer
                        .remove_block(v_cursor_start, v_cursor_end);

                    //TODO: ADD block content to editor.yank_buffer too

                    editor.cursor = v_block.start;
                    editor.undo_actions.push(Action::UndoDeleteBlock(
                        OldCursorPosition::new(v_block.start, c_mut_viewport.top),
                        block_content,
                    ));
                    editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
                }
            }

            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_deletion {
    use crate::{
        buff::Buffer,
        editor::{
            core::{actions::action::Action, mode::Mode},
            Editor,
        },
    };
    use std::io::{Cursor, Seek, Write};
    use tempfile::NamedTempFile;

    fn setup_temp_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let content = "Line1\nLine2\nLine3";
        temp_file
            .write_all(content.as_bytes())
            .expect("Failed to write");
        temp_file.flush().expect("Failed to flush");
        temp_file
            .seek(std::io::SeekFrom::Start(0))
            .expect("Failed to seek");
        temp_file
    }

    fn mock_file_editor() -> Editor<Cursor<Vec<u8>>> {
        let tmp_file = setup_temp_file();
        let path = tmp_file.path().to_str().unwrap().to_string();
        let mut editor = Editor::default();
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(path));
        editor
    }

    #[test]
    fn test_remove_char_at() {
        let mut editor = mock_file_editor();
        editor.cursor = (2, 0); // Set cursor to 'n' in "Line1"
        Action::RemoveCharAt.execute(&mut editor).unwrap();

        let line = editor.viewports.c_viewport().buffer.get(0).unwrap();
        assert_eq!(line, "Lie1"); // 'n' should be removed
    }

    #[test]
    fn test_remove_char() {
        let mut editor = mock_file_editor();
        editor.cursor = (5, 0); // Cursor at the end of "Line1"
        Action::RemoveChar.execute(&mut editor).unwrap();
        let line = editor.viewports.c_viewport().buffer.get(0).unwrap();
        assert_eq!(line, "Line"); // '1' should be removed
    }

    #[test]
    fn test_delete_line() {
        let mut editor = mock_file_editor();
        editor.cursor.1 = 1; // Delete "Line2"
        Action::DeleteLine.execute(&mut editor).unwrap();
        let buffer = &editor.viewports.c_viewport().buffer.lines;
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], "Line1");
        assert_eq!(buffer[1], "Line3");
    }

    #[test]
    fn test_remove_char_from_search() {
        let mut editor = mock_file_editor();
        editor.search = "hello".to_string();
        Action::RemoveCharFrom(true).execute(&mut editor).unwrap();
        assert_eq!(editor.search, "hell"); // Last char removed
    }

    #[test]
    fn test_remove_char_from_command() {
        let mut editor = mock_file_editor();
        editor.command = ":wq".to_string();
        Action::RemoveCharFrom(false).execute(&mut editor).unwrap();
        assert_eq!(editor.command, ":w"); // Last char removed
    }

    #[test]
    fn test_delete_block() {
        let mut editor = mock_file_editor();
        editor.mode = Mode::Visual;
        editor.visual_cursor = Some((4, 1));
        Action::DeleteBlock.execute(&mut editor).unwrap();
        let buffer = &editor.viewports.c_viewport().buffer.lines;
        assert_eq!(buffer.len(), 1); // Only one line remains
        assert_eq!(buffer[0], "Line3"); // The selected block was deleted
    }
}
