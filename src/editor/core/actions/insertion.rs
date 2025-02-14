use std::io::Write;

use crate::editor::{core::mode::Mode, Editor};

use super::action::{Action, OldCursorPosition};

impl Action {
    pub fn insertion<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::AddStr(s) => {
                let len = s.len();
                let cursor_viewport = editor.v_cursor();
                editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .add_str(s.clone(), cursor_viewport);
                editor.undo_insert_actions.push(Action::UndoStrAt(
                    OldCursorPosition::new(editor.cursor, editor.viewports.c_viewport().top),
                    cursor_viewport,
                    len,
                ));
                editor.cursor.0 += len as u16;
            }
            Action::AddChar(c) => {
                let cursor_viewport = editor.v_cursor();
                editor.undo_insert_actions.push(Action::UndoCharAt(
                    OldCursorPosition::new(editor.cursor, editor.viewports.c_viewport().top),
                    cursor_viewport,
                ));

                editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .add_char(*c, cursor_viewport);
                editor.cursor.0 += 1;
            }
            Action::AddCommandChar(c) => editor.command.push(*c),
            // the modal should have a push fn
            Action::AddModalChar(c) => {
                if let Some(ref mut modal) = editor.modal {
                    modal.push(*c);
                }
            }

            Action::AddSearchChar(c) => {
                editor.search.push(*c);
                editor.buffer_actions.push(Action::FindSearchValue)
            }

            Action::NewLineInsertionAtCursor => {
                let v_cursor = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();

                editor.cursor.0 = current_viewport.buffer.new_line(v_cursor);
                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(OldCursorPosition::new(
                        editor.cursor,
                        current_viewport.top,
                    )));
            }

            Action::NewLineInsertionBelowCursor => {
                let (v_x, v_y) = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();

                editor.cursor.0 = current_viewport.buffer.new_line((v_x, v_y + 1));
                editor.move_next_line();

                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(OldCursorPosition::new(
                        editor.cursor,
                        editor.viewports.c_viewport().top,
                    )));
            }

            Action::NewLine => {
                let (v_x, v_y) = editor.v_cursor();
                let indentation = editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .new_line_with_text((v_x, v_y));

                editor.cursor.0 = indentation;
                editor.move_next_line();

                editor.undo_actions.push(Action::UndoNewLineWithText(
                    OldCursorPosition::new(
                        (editor.cursor.0, editor.cursor.1.saturating_sub(1)),
                        editor.viewports.c_viewport().top,
                    ),
                    indentation as usize,
                ));
            }

            _ => {}
        }

        Ok(())
    }
}
