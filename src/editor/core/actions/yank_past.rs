use crate::editor::{core::mode::Mode, CursorBlock, Editor};

use super::action::Action;

impl Action {
    pub fn yank_past(&self, editor: &mut Editor) -> anyhow::Result<()> {
        match self {
            Action::YankLine => {
                let current_viewport = editor.viewports.c_mut_viewport();
                let (_, y) = current_viewport.viewport_cursor(&editor.cursor);
                editor.yank_buffer = vec![current_viewport.buffer.get(y as usize)];
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }
            Action::YankBlock => {
                if let Some(v_block) = editor.get_visual_block_pos() {
                    let c_mut_viewport = editor.viewports.c_mut_viewport();
                    editor.yank_buffer = c_mut_viewport.buffer.get_block(
                        c_mut_viewport.viewport_cursor(&v_block.start),
                        c_mut_viewport.viewport_cursor(&v_block.end),
                    );
                }
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }
            Action::Past => {
                let content = editor.yank_buffer.clone();
                let current_viewport = editor.viewports.c_mut_viewport();
                let start = current_viewport.viewport_cursor(&editor.cursor);
                let mut y = start.1 as usize;
                let mut start_x = 0; // allow us know if line is empty and for undoPast know where
                                     // the past start

                let mut end_y = 0; // allow us to know where the past end for undo block
                let mut end_x = 0; // same as upper comment

                for (i, c) in content.iter().enumerate() {
                    if let Some(line) = c {
                        match i {
                            _ if i == 0 => {
                                let mut x = start.0 as usize;
                                if i == content.len() - 1 {
                                    end_x = line.len() - 1;
                                }

                                // because if the buffer line is an empty line it will crash the
                                // app
                                if let Some(buffer_line) = current_viewport.buffer.get(y) {
                                    start_x = buffer_line.len();
                                    if start_x > 0 {
                                        x += 1;
                                    }
                                }
                                current_viewport.buffer.insert_str(y, x, line)
                            }
                            _ => {
                                if i == content.len() - 1 {
                                    end_x = line.len() - 1;
                                }
                                current_viewport.buffer.push_or_insert(line.clone(), y)
                            }
                        }
                    }
                    end_y += 1;
                    y += 1;
                }

                editor.undo_actions.push(Action::UndoPast(
                    CursorBlock {
                        start: (start_x as u16, editor.cursor.1),
                        end: (end_x as u16, editor.cursor.1 + end_y - 1),
                    },
                    current_viewport.top,
                ));
            }
            _ => {}
        }

        Ok(())
    }
}
