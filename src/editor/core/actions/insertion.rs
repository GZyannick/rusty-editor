use crate::editor::{core::mode::Mode, Editor};

use super::action::{Action, OldCursorPosition};

impl Action {
    pub fn insertion(&self, editor: &mut Editor) -> anyhow::Result<()> {
        match self {
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

            Action::AddSearchChar(c) => {
                editor.search.push(*c);
                editor.buffer_actions.push(Action::FindSearchValue)
            }

            Action::NewLineInsertionAtCursor => {
                let v_cursor = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();

                current_viewport.buffer.new_line(v_cursor, false);

                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));
                editor.cursor.0 = 0;

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

                current_viewport.buffer.new_line((v_x, v_y + 1), false);
                editor.move_next_line();
                editor.cursor.0 = 0;

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
                editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .new_line((v_x, v_y), false);
                editor.cursor.0 = 0;
                editor.move_next_line();

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(OldCursorPosition::new(
                        (editor.cursor.0, editor.cursor.1.saturating_sub(1)),
                        editor.viewports.c_viewport().top,
                    )));
            }

            _ => {}
        }

        Ok(())
    }
}
