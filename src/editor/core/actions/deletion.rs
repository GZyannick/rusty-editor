use std::io::Write;

use crate::{
    editor::{core::mode::Mode, Editor},
    helper::clipboard::copy_to_clipboard,
};

use super::action::{Action, OldCursorPosition};

impl Action {
    pub fn deletion<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::RemoveCharAt(cursor) => {
                if editor.viewports.c_viewport().get_line_len(cursor) > 0 {
                    editor
                        .viewports
                        .c_mut_viewport()
                        .buffer
                        .remove_char(*cursor);
                }
            }

            Action::RemoveModalChar => {
                if let Some(ref mut modal) = editor.modal {
                    modal.pop();
                }
            }
            Action::RemoveChar => {
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
                        let new_x_pos =
                            current_viewport.get_line_len(&(editor.cursor.0, editor.cursor.1 - 1));
                        current_viewport.buffer.remove_char_line(cursor_viewport);
                        editor.move_prev_line();
                        editor.cursor.0 = new_x_pos;
                    }
                    _ => {}
                }
            }
            Action::DeleteLine => {
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
