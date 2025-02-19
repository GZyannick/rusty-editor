use std::io::Write;

use crate::{
    editor::{core::mode::Mode, CursorBlock, Editor},
    helper::clipboard,
};

use super::action::Action;

impl Action {
    pub fn yank_past<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::YankLine => {
                let current_viewport = editor.viewports.c_mut_viewport();
                let (_, y) = current_viewport.viewport_cursor(&editor.cursor);
                if let Some(str) = current_viewport.buffer.get(y as usize) {
                    clipboard::copy_to_clipboard(&str);
                }
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }

            Action::YankBlock => {
                if let Some(v_block) = editor.get_visual_block_pos() {
                    let c_mut_viewport = editor.viewports.c_mut_viewport();
                    let to_copy = c_mut_viewport.buffer.get_block(
                        c_mut_viewport.viewport_cursor(&v_block.start),
                        c_mut_viewport.viewport_cursor(&v_block.end),
                    );
                    if let Some(str) = to_copy {
                        clipboard::copy_to_clipboard(&str);
                    }
                }
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }

            Action::Past => {
                if let Some(content) = clipboard::paste_from_clipboard() {
                    let current_viewport = editor.viewports.c_mut_viewport();
                    let v_cursor = current_viewport.viewport_cursor(&editor.cursor);
                    let mut end_x: usize = 0;
                    let mut start_x: usize = 0;
                    let mut y: usize = v_cursor.1 as usize;
                    let mut end_y: u16 = 0; // we can know where the past end
                    let mut start_y: usize = 0;
                    let mut remove_past_line = true;

                    for (i, line) in content.iter().enumerate() {
                        match i {
                            _ if i == 0 => {
                                let line_len = current_viewport.get_line_len(&editor.cursor);

                                (start_x, end_x) = match line_len > 0 {
                                    true => {
                                        (v_cursor.0 as usize + 1, v_cursor.0 as usize + line.len())
                                    }
                                    false => {
                                        remove_past_line = false;
                                        (
                                            v_cursor.0 as usize,
                                            v_cursor.0 as usize + line.len().saturating_sub(1),
                                        )
                                    }
                                };

                                // if we past multi line and we are not at the end of line
                                // we past to the next line
                                match content.len() > 1 && start_x < line_len as usize {
                                    true => {
                                        start_y += 1;
                                        start_x = 0;
                                        current_viewport
                                            .buffer
                                            .push_or_insert(line.clone(), y + start_y)
                                    }
                                    false => {
                                        current_viewport.buffer.insert_str(
                                            v_cursor.1 as usize,
                                            start_x,
                                            line,
                                        );
                                    }
                                }
                            }
                            _ => {
                                if i == content.len().saturating_sub(1) && !line.is_empty() {
                                    end_x = line.len().saturating_sub(1);
                                }
                                current_viewport
                                    .buffer
                                    .push_or_insert(line.clone(), y + start_y)
                            }
                        }
                        y += 1;
                        end_y += 1;
                    }

                    editor.undo_actions.push(Action::UndoPast(
                        CursorBlock {
                            start: (start_x as u16, editor.cursor.1 + start_y as u16),
                            end: (
                                end_x as u16,
                                editor.cursor.1 + start_y as u16 + end_y.saturating_sub(1),
                            ),
                        },
                        current_viewport.top,
                        remove_past_line,
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
