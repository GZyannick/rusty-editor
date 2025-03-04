use std::io::Write;

use crate::editor::{core::chartype::CharType, Editor, TERMINAL_LINE_LEN_MINUS};

use super::action::Action;

impl Action {
    pub fn movement<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::MoveUp => {
                editor.move_prev_line();
            }
            Action::MoveRight => {
                // we clear the buffer because to overwrite it if needed;
                // if we are at the end of the line_len - 1 move to next line
                if editor.get_specific_line_len_by_mode() > editor.cursor.0 {
                    editor.clear_buffer_x_cursor();
                    editor.cursor.0 += 1;
                }
            }
            // we clear the buffer because to overwrite it if needed;
            Action::MoveLeft => {
                if editor.cursor.0 > 0 {
                    editor.clear_buffer_x_cursor();
                    editor.cursor.0 -= 1;
                }
            }

            Action::MoveDown => editor.move_next_line(),

            Action::PageUp => editor.viewports.c_mut_viewport().page_up(),

            Action::StartOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 = 0;
            }

            Action::EndOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 = editor
                    .viewports
                    .c_viewport()
                    .get_line_len(&editor.cursor)
                    .wrapping_sub(TERMINAL_LINE_LEN_MINUS)
            }

            Action::PageDown => editor.viewports.c_mut_viewport().page_down(&editor.cursor),

            Action::StartOfFile => {
                editor.viewports.c_mut_viewport().move_top();
                editor.cursor.1 = 0;
            }

            Action::EndOfFile => {
                editor
                    .viewports
                    .c_mut_viewport()
                    .move_end(&mut editor.cursor);
            }
            Action::CenterLine => {
                editor
                    .viewports
                    .c_mut_viewport()
                    .center_line(&mut editor.cursor);
            }

            Action::MoveNext => {
                editor.clear_buffer_x_cursor();
                let current_viewport = editor.viewports.c_viewport();
                let v_cursor = editor.v_cursor();

                if let Some(line) = current_viewport.buffer.get(v_cursor.1 as usize) {
                    let base_len = line.len().saturating_sub(1) as u16;
                    let line = line[v_cursor.0 as usize..].to_string();
                    if line.len() > 1 {
                        CharType::goto_diff_type(line, Some(base_len), &mut editor.cursor.0);
                        // CharType.goto_diff_type(&line, Some(base_len), &mut editor.cursor.0);
                    } else if current_viewport.buffer.lines.len() - 1 > v_cursor.1 as usize {
                        editor.cursor.0 = 0;
                        editor.move_next_line();
                    }
                }
            }
            Action::MovePrev => {
                editor.clear_buffer_x_cursor();
                let current_viewport = editor.viewports.c_viewport();
                let v_cursor = editor.v_cursor();
                if let Some(line) = current_viewport.buffer.get(v_cursor.1 as usize) {
                    if line.is_empty() && v_cursor.1 == 0 {
                        return Ok(());
                    } else if line.is_empty() && v_cursor.1 > 0 {
                        if let Some(prev_line) =
                            current_viewport.buffer.get(v_cursor.1 as usize - 1)
                        {
                            editor.cursor.0 = prev_line.len().saturating_sub(1) as u16;

                            editor.move_prev_line();
                        }
                    } else {
                        let line = line[..=v_cursor.0 as usize].to_string();
                        if line.len() > 1 {
                            CharType::goto_diff_type(line, None, &mut editor.cursor.0);
                        } else if v_cursor.1 > 0 {
                            if let Some(prev_line) =
                                current_viewport.buffer.get(v_cursor.1 as usize - 1)
                            {
                                editor.cursor.0 = prev_line.len().saturating_sub(1) as u16;
                                editor.move_prev_line();
                            }
                        }
                    }
                }
            }

            Action::GotoPos(new_cursor_pos) => {
                let current_viewport = editor.viewports.c_mut_viewport();
                if new_cursor_pos.1 as usize > current_viewport.get_buffer_len() {
                    return Ok(());
                }
                editor.cursor = current_viewport.move_to(new_cursor_pos);
                editor.buffer_actions.push(Action::CenterLine);
            }

            _ => {}
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests_movement {
    use tempfile::NamedTempFile;

    use crate::{
        buff::Buffer,
        editor::{core::actions::action::Action, Editor},
    };
    use std::io::{Cursor, Seek, Write};

    fn setup_temp_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().expect("Failed to create a temporary_file");
        let content =
            "USE\nThis is a test file with multiple line.\nHere is a keyword we will search.\nAnother line with keyword.";
        temp_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");

        // return the cursor at 0:0
        temp_file
            .seek(std::io::SeekFrom::Start(0))
            .expect("Failed to seek temp file");
        temp_file
    }

    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::default()
    }

    fn mock_file_editor() -> Editor<Cursor<Vec<u8>>> {
        let tmp_file = setup_temp_file();
        let path = tmp_file.path().to_str().unwrap().to_string();
        let mut editor = create_mock_editor();
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(path));
        editor
    }

    #[test]
    fn test_move_down() {
        let mut editor = mock_file_editor();
        let old_cursor = editor.cursor;
        Action::MoveDown.execute(&mut editor).unwrap();

        assert!(
            old_cursor.1 + 1 == editor.cursor.1,
            "cursor.1 should be superior by 1"
        );
        editor.cursor.1 = 3;

        Action::MoveDown.execute(&mut editor).unwrap();
        assert!(
            3 == editor.cursor.1,
            "cursor.1 should still be 3 because cursor cannot be > to file_len"
        );
    }

    #[test]
    fn test_move_up() {
        let mut editor = mock_file_editor();
        let old_cursor = editor.cursor;
        Action::MoveUp.execute(&mut editor).unwrap();

        assert!(
            old_cursor.1 == editor.cursor.1,
            "cursor.1 should still be the same"
        );
        editor.cursor.1 = 3;
        Action::MoveUp.execute(&mut editor).unwrap();
        assert!(2 == editor.cursor.1, "cursor.1 should be inferior by 1");
    }

    #[test]
    fn test_move_left() {
        let mut editor = mock_file_editor();
        let old_cursor = editor.cursor;
        Action::MoveLeft.execute(&mut editor).unwrap();

        assert!(
            old_cursor.0 == editor.cursor.0,
            "cursor.0 should still be the same"
        );
        editor.cursor.0 = 3;
        Action::MoveLeft.execute(&mut editor).unwrap();
        assert!(2 == editor.cursor.0, "cursor.0 should be inferior by 1");
    }
    #[test]
    fn test_move_right() {
        let mut editor = mock_file_editor();
        let old_cursor = editor.cursor;
        Action::MoveRight.execute(&mut editor).unwrap();

        assert!(
            old_cursor.0 + 1 == editor.cursor.0,
            "cursor.0 should be superior by 1"
        );
        editor.cursor.0 = 3;
        Action::MoveRight.execute(&mut editor).unwrap();
        assert!(3 == editor.cursor.0, "cursor.0 should still be the same");
    }

    #[test]
    fn test_start_and_end_of_line() {
        let mut editor = mock_file_editor();
        Action::EndOfLine.execute(&mut editor).unwrap();
        let ll = editor.viewports.c_viewport().get_line_len(&editor.cursor);

        assert!(
            ll - 1 == editor.cursor.0,
            "cursor.0 should be at the end of line"
        );

        Action::StartOfLine.execute(&mut editor).unwrap();

        assert!(
            0 == editor.cursor.0,
            "cursor.0 should be at the start of line"
        );
    }
    #[test]
    fn test_start_and_end_of_file() {
        let mut editor = mock_file_editor();
        Action::EndOfFile.execute(&mut editor).unwrap();
        let bl = editor.viewports.c_viewport().get_buffer_len();

        assert!(
            bl as u16 - 1 == editor.cursor.1,
            "cursor.0 should be at the end of file"
        );

        Action::StartOfFile.execute(&mut editor).unwrap();

        assert!(
            0 == editor.cursor.1,
            "cursor.0 should be at the start of file"
        );
    }

    #[test]
    fn test_goto_pos() {
        let mut editor = mock_file_editor();
        Action::GotoPos((0, 20)).execute(&mut editor).unwrap();
        assert!(editor.cursor == (0, 0), "cursor should not move");

        Action::GotoPos((0, 2)).execute(&mut editor).unwrap();
        assert!(editor.cursor == (0, 1), "cursor should be at 0, 2");
    }

    #[test]
    fn test_move_next() {
        let mut editor = mock_file_editor();

        // set cursor where there are multiple word on the line
        editor.cursor = (0, 1);
        Action::MoveNext.execute(&mut editor).unwrap();

        assert!(
            editor.cursor.0 > 1,
            "Cursor should move to the start of the next word"
        );
    }

    #[test]
    fn test_move_prev() {
        let mut editor = mock_file_editor();

        editor.cursor = (10, 1); // Assuming cursor is inside "keyword"

        Action::MovePrev.execute(&mut editor).unwrap();

        assert!(
            editor.cursor.0 < 10,
            "Cursor should move to the start of the previous word"
        );
    }
}
