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
                editor
                    .undo_insert_actions
                    .push(Action::UndoCharAt(OldCursorPosition::new(
                        editor.cursor,
                        editor.viewports.c_viewport().top,
                    )));

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
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
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
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
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
                if !editor.is_viewport_modifiable() {
                    return Ok(());
                }
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
#[cfg(test)]
mod tests_insertion {
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
    fn test_add_str() {
        let mut editor = mock_file_editor();

        Action::AddStr("Hello".to_string())
            .execute(&mut editor)
            .unwrap();

        let line = editor.viewports.c_viewport().buffer.get(0).unwrap();
        assert_eq!(line, "HelloLine1");
    }

    #[test]
    fn test_add_char() {
        let mut editor = mock_file_editor();
        let initial_cursor = editor.cursor;

        Action::AddChar('X').execute(&mut editor).unwrap();

        assert_eq!(editor.cursor.0, initial_cursor.0 + 1);
        assert_eq!(
            editor
                .viewports
                .c_viewport()
                .buffer
                ._get_char(&initial_cursor),
            Some('X')
        );
    }

    #[test]
    fn test_add_command_char() {
        let mut editor = mock_file_editor();
        Action::AddCommandChar('X').execute(&mut editor).unwrap();

        assert_eq!(editor.command, "X");
    }

    #[test]
    fn test_add_search_char() {
        let mut editor = mock_file_editor();
        Action::AddSearchChar('Z').execute(&mut editor).unwrap();
        assert_eq!(editor.search, "Z");
    }

    #[test]
    fn test_new_line_insertion_at_cursor() {
        let mut editor = mock_file_editor();
        let initial_cursor = editor.cursor;
        let initial_buffer_len = editor.viewports.c_viewport().get_buffer_len();

        Action::NewLineInsertionAtCursor
            .execute(&mut editor)
            .unwrap();

        assert_eq!(editor.cursor, initial_cursor);
        assert_eq!(
            editor.viewports.c_viewport().get_buffer_len(),
            initial_buffer_len + 1
        );
        assert!(
            matches!(editor.mode, Mode::Insert),
            "should be in insert mode"
        );
    }

    #[test]
    fn test_new_line_insertion_below_cursor() {
        let mut editor = mock_file_editor();
        let initial_cursor = editor.cursor;

        Action::NewLineInsertionBelowCursor
            .execute(&mut editor)
            .unwrap();

        assert_eq!(editor.cursor.0, 0);
        assert_eq!(editor.cursor.1, initial_cursor.1 + 1);
        assert!(
            matches!(editor.mode, Mode::Insert),
            "should be in insert mode"
        );
    }

    #[test]
    fn test_new_line() {
        let mut editor = mock_file_editor();
        let initial_cursor = editor.cursor;

        Action::NewLine.execute(&mut editor).unwrap();

        assert_eq!(editor.cursor.0, 0);
        assert_eq!(editor.cursor.1, initial_cursor.1 + 1);
    }
}
