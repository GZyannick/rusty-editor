use crate::{
    editor::{core::mode::Mode, CursorBlock, Editor},
    helper::clipboard,
    log_message,
};

use super::action::Action;

impl Action {
    pub fn yank_past(&self, editor: &mut Editor) -> anyhow::Result<()> {
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
                    let start = current_viewport.viewport_cursor(&editor.cursor);
                    let mut y = start.1 as usize;
                    let mut start_x: u16 = 0; // allow us know if line is empty and for undoPast know where
                                              // the past start

                    let mut end_y: u16 = 0; // allow us to know where the past end for undo block
                    let mut end_x: u16 = 0; // same as upper comment
                    for (i, line) in content.iter().enumerate() {
                        match i {
                            _ if i == 0 => {
                                let mut x = start.0 as usize;
                                if i == content.len() - 1 {
                                    end_x = line.len() as u16 - 1;
                                }

                                // because if the buffer line is an empty line it will crash the
                                // app
                                if let Some(buffer_line) = current_viewport.buffer.get(y) {
                                    start_x = buffer_line.len() as u16;
                                    if start_x > 0 {
                                        x += 1;
                                    }
                                }
                                current_viewport.buffer.insert_str(y, x, line)
                            }
                            _ => {
                                if i == content.len() - 1 && !line.is_empty() {
                                    end_x = line.len() as u16 - 1;
                                }
                                current_viewport.buffer.push_or_insert(line.clone(), y)
                            }
                        }
                        end_y += 1;
                        y += 1;
                    }
                    editor.undo_actions.push(Action::UndoPast(
                        CursorBlock {
                            start: (start_x, editor.cursor.1),
                            end: (end_x, editor.cursor.1 + end_y.saturating_sub(1)),
                        },
                        current_viewport.top,
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}
