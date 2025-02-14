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
