use crate::{editor::Editor, log_message};

use super::action::Action;

impl Action {
    pub fn undo(&self, editor: &mut Editor) -> anyhow::Result<()> {
        match self {
            Action::UndoCharAt(old_cursor, v_cursor) => {
                editor.buffer_actions.push(Action::RemoveCharAt(*v_cursor));
                editor.cursor = old_cursor.cursor;
            }

            Action::UndoStrAt(old_cursor, v_cursor, str_len) => {
                if let Some(line) = editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .lines
                    .get_mut(v_cursor.1 as usize)
                {
                    line.drain(v_cursor.0 as usize..v_cursor.0 as usize + *str_len);
                    editor.cursor = old_cursor.cursor
                };
            }

            Action::Undo => {
                if let Some(action) = editor.undo_actions.pop() {
                    action.execute(editor)?;
                }
            }

            Action::UndoDeleteLine(old_cursor, Some(content)) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let current_viewport = editor.viewports.c_mut_viewport();
                let buffer_len = current_viewport.get_buffer_len();
                if cy as usize >= buffer_len {
                    current_viewport.buffer.lines.push(content.clone());
                    editor.cursor.1 += 1;
                } else {
                    current_viewport
                        .buffer
                        .lines
                        .insert(cy as usize, content.clone());
                }
                current_viewport.top = old_cursor.top;
                editor.cursor.1 = old_cursor.cursor.1;

                // put the line at the center of screen if possible
                editor.buffer_actions.push(Action::CenterLine)
            }

            Action::UndoNewLine(old_cursor) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let c_mut_viewport = editor.viewports.c_mut_viewport();
                c_mut_viewport.buffer.remove(cy as usize);
                c_mut_viewport.top = old_cursor.top;
                editor.cursor.1 = old_cursor.cursor.1;
            }

            Action::UndoNewLineWithText(old_cursor) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let c_mut_viewport = editor.viewports.c_mut_viewport();
                let mut buffer_line = String::new();

                // get the y + 1 line to copy and remove it;
                if let Some(line) = c_mut_viewport.buffer.lines.get(cy as usize + 1) {
                    buffer_line = line.clone();
                    c_mut_viewport.buffer.remove(cy as usize + 1);
                }
                // push the content of y + 1 in y
                if let Some(line) = c_mut_viewport.buffer.lines.get_mut(cy as usize) {
                    line.push_str(&buffer_line);
                }

                c_mut_viewport.top = old_cursor.top;
                editor.cursor.1 = old_cursor.cursor.1;
            }

            Action::UndoMultiple(actions) => {
                for action in actions.iter().rev() {
                    action.execute(editor)?;
                }
            }

            Action::UndoDeleteBlock(start, content) => {
                let start_y = start.cursor.1 + start.top;
                let current_viewport = editor.viewports.c_mut_viewport();
                let mut y = start_y as usize;

                // this if could be deleted but there is
                // no need to run a big iterator when the size = 1
                if content.len() == 1 {
                    if let Some(line) = content.first().unwrap() {
                        current_viewport
                            .buffer
                            .insert_str(y, start.cursor.0 as usize, line);
                    }
                } else {
                    for (i, c) in content.iter().enumerate() {
                        if let Some(line) = c {
                            // handle the first if she need to be insert in a existing line
                            let len = current_viewport.get_line_len(&(start.cursor.0, y as u16));
                            match i {
                                _ if i == 0 && len > 0 && start.cursor.0 == len => current_viewport
                                    .buffer
                                    .insert_str(y, start.cursor.0 as usize, line),
                                _ if i == content.len() - 1 => match line.contains('\n') {
                                    true => current_viewport
                                        .buffer
                                        .push_or_insert(line[0..line.len() - 1].to_string(), y),
                                    false => current_viewport.buffer.insert_str(y, 0, line),
                                },
                                _ => current_viewport.buffer.push_or_insert(line.clone(), y),
                            }
                        }
                        y += 1;
                    }
                }

                current_viewport.top = start.top;
                editor.cursor.1 = start.cursor.1;
                editor.buffer_actions.push(Action::CenterLine)
            }
            Action::UndoPast(cursor, top, remove_past_line) => {
                let current_viewport = editor.viewports.c_mut_viewport();
                let start_y = cursor.start.1 + top;
                let end_y = cursor.end.1 + top;
                current_viewport
                    .buffer
                    .remove_block((cursor.start.0, start_y), (cursor.end.0, end_y));

                if !remove_past_line {
                    current_viewport.buffer.new_line((0, start_y));
                }
                current_viewport.top = *top;
                editor.cursor.1 = cursor.start.1;
                editor.buffer_actions.push(Action::CenterLine);
            }

            _ => {}
        }
        Ok(())
    }
}
