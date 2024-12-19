use std::fs::metadata;

use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use crate::buff::Buffer;
use crate::editor::ui::clear::ClearDraw;
use crate::editor::{MOVE_PREV_OR_NEXT_LINE, TERMINAL_LINE_LEN_MINUS};
use crate::viewport::Viewport;

use super::super::Editor;
use super::command::Command;
use super::mode::Mode;

#[derive(Debug, Clone)]
pub struct OldCursorPosition {
    pub cursor: (u16, u16),
    pub top: u16,
}

impl OldCursorPosition {
    pub fn new(cursor: (u16, u16), top: u16) -> Self {
        OldCursorPosition { cursor, top }
    }
}

impl ClearDraw for Viewport {}

#[derive(Debug, Clone)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    RemoveChar,
    RemoveCharAt((u16, u16)),
    WaitingCmd(char),
    DeleteLine,
    DeleteWord,
    AddCommandChar(char),
    NewLine,
    PageDown,
    PageUp,
    EndOfLine,
    StartOfLine,
    SaveFile,
    EndOfFile,
    StartOfFile,
    CenterLine,
    Undo,
    Quit,
    NewLineInsertionBelowCursor,
    NewLineInsertionAtCursor,

    // TODO: later add a way to use command and use :13 to move to line and dont pass it args
    UndoDeleteLine(OldCursorPosition, Option<String>), //cursor.1 , top, content
    UndoNewLine(OldCursorPosition),
    UndoMultiple(Vec<Action>),
    UndoCharAt(OldCursorPosition, (u16, u16)),
    ExecuteCommand,
    RemoveCommandChar,
    EnterFileOrDirectory,
    SwapViewportToExplorer,
    SwapViewportToPopupExplorer,
}

impl Action {
    pub fn execute(&self, editor: &mut Editor) -> anyhow::Result<()> {
        match self {
            Action::Quit => (),
            Action::MoveUp => {
                editor.move_prev_line();
            }

            Action::MoveRight => {
                // we clear the buffer because to overwrite it if needed;
                editor.clear_buffer_x_cursor();
                // if we are at the end of the line_len - 1 move to next line
                let line_len = editor.get_specific_line_len_by_mode();
                match line_len > editor.cursor.0 {
                    true => editor.cursor.0 += 1,
                    false if MOVE_PREV_OR_NEXT_LINE => {
                        // if we are at the end of the line go ot the next line if exist
                        // and move the cursor to the start of the line
                        editor.move_next_line();
                        editor.cursor.0 = 0;
                    }
                    false => (),
                }
            }
            Action::MoveLeft => {
                // we clear the buffer because to overwrite it if needed;
                editor.clear_buffer_x_cursor();
                if editor.cursor.0 > 0 {
                    editor.cursor.0 -= 1;
                } else if editor.cursor.0 == 0
                    && (editor.cursor.1 > 0 || editor.c_viewport().top > 0)
                    && MOVE_PREV_OR_NEXT_LINE
                {
                    // if we are at the start of the line go ot the prev line if exist
                    // and move the cursor to the end of the line
                    editor.move_prev_line();
                    editor.cursor.0 = editor.c_viewport().get_line_len(&editor.cursor);
                }
            }

            Action::MoveDown => {
                editor.move_next_line();
            }
            Action::AddChar(c) => {
                let cursor_viewport = editor.v_cursor();
                // editor.undo_insert_actions();
                editor.undo_insert_actions.push(Action::UndoCharAt(
                    OldCursorPosition::new(editor.cursor, editor.c_viewport().top),
                    cursor_viewport,
                ));

                editor.c_mut_viewport().buffer.add_char(*c, cursor_viewport);
                editor.cursor.0 += 1;
            }
            Action::RemoveCharAt(cursor) => {
                if editor.c_viewport().get_line_len(cursor) > 0 {
                    editor.c_mut_viewport().buffer.remove_char(*cursor);
                }
            }
            Action::UndoCharAt(old_cursor, v_cursor) => {
                editor.buffer_actions.push(Action::RemoveCharAt(*v_cursor));
                editor.cursor = old_cursor.cursor;
            }
            Action::RemoveChar => {
                let cursor_viewport = editor.v_cursor();
                match cursor_viewport.0 > 0 {
                    true => {
                        editor.cursor.0 -= 1;
                        editor
                            .c_mut_viewport()
                            .buffer
                            .remove_char((cursor_viewport.0 - 1, cursor_viewport.1));
                    }
                    false if cursor_viewport.1 > 0 => {
                        // we get the size of the prev line before change
                        // because we want the text that will be added behind the cursor
                        let new_x_pos = editor
                            .c_viewport()
                            .get_line_len(&(editor.cursor.0, editor.cursor.1 - 1));
                        editor
                            .c_mut_viewport()
                            .buffer
                            .remove_char_line(cursor_viewport);
                        editor.move_prev_line();
                        editor.cursor.0 = new_x_pos;
                    }
                    _ => {}
                }
            }
            Action::EnterMode(mode) => {
                // if we enter insert mode
                if !matches!(editor.mode, Mode::Insert) && matches!(mode, Mode::Insert) {
                    editor.stdout.execute(cursor::SetCursorStyle::SteadyBar)?;
                    editor.undo_insert_actions = vec![];
                }

                // if we leave insert mode
                if matches!(editor.mode, Mode::Insert) && !matches!(mode, Mode::Insert) {
                    editor.stdout.execute(cursor::SetCursorStyle::SteadyBlock)?;
                    if !editor.undo_insert_actions.is_empty() {
                        let actions = std::mem::take(&mut editor.undo_insert_actions);
                        editor.undo_actions.push(Action::UndoMultiple(actions));
                    }
                }

                if matches!(editor.mode, Mode::Command) && !matches!(mode, Mode::Command) {
                    editor.command = String::new();
                }

                editor.mode = *mode;
            }
            Action::AddCommandChar(c) => {
                editor.command.push(*c);
            }
            Action::NewLineInsertionAtCursor => {
                let v_cursor = editor.v_cursor();
                editor.c_mut_viewport().buffer.new_line(v_cursor, false);
                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));
                editor.cursor.0 = 0;

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(OldCursorPosition::new(
                        editor.cursor,
                        editor.c_viewport().top,
                    )));
            }
            Action::NewLineInsertionBelowCursor => {
                let (v_x, v_y) = editor.v_cursor();
                editor
                    .c_mut_viewport()
                    .buffer
                    .new_line((v_x, v_y + 1), false);
                editor.move_next_line();
                editor.cursor.0 = 0;

                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(OldCursorPosition::new(
                        editor.cursor,
                        editor.c_viewport().top,
                    )));
            }
            Action::NewLine => {
                let (v_x, v_y) = editor.v_cursor();
                editor
                    .c_mut_viewport()
                    .buffer
                    .new_line((v_x, v_y + 1), false);
                editor.cursor.0 = 0;
                editor.move_next_line();
            }
            Action::SaveFile => {
                editor.c_mut_viewport().buffer.save()?;
            }
            Action::PageUp => {
                editor.c_mut_viewport().page_up();
            }
            Action::StartOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 = 0;
            }
            Action::EndOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 =
                    editor.c_viewport().get_line_len(&editor.cursor) - TERMINAL_LINE_LEN_MINUS;
            }
            Action::PageDown => {
                let cursor = &editor.cursor.clone(); // TO REFACTO DEV_ERROR
                editor.c_mut_viewport().page_down(cursor);
            }
            Action::WaitingCmd(c) => {
                editor
                    .stdout
                    .queue(cursor::SetCursorStyle::BlinkingUnderScore)?;
                editor.waiting_command = Some(*c);
            }
            Action::DeleteLine => {
                let (_, y) = editor.v_cursor();
                let content = editor.c_viewport().buffer.get(y as usize).clone();
                editor.c_mut_viewport().buffer.remove(y as usize);

                editor.undo_actions.push(Action::UndoDeleteLine(
                    OldCursorPosition::new(editor.cursor, editor.c_viewport().top),
                    content,
                ));
            }
            Action::DeleteWord => {
                let v_cursor = editor.v_cursor();
                editor.c_mut_viewport().buffer.remove_word(v_cursor)
            }
            Action::StartOfFile => {
                editor.c_mut_viewport().move_top();
                editor.cursor.1 = 0;
            }
            Action::EndOfFile => {
                // DEV_ERROR
                let mut cursor = editor.cursor.clone();
                editor.c_mut_viewport().move_end(&mut cursor);
                editor.cursor = cursor;
            }
            Action::Undo => {
                if let Some(action) = editor.undo_actions.pop() {
                    action.execute(editor)?;
                }
            }
            Action::UndoDeleteLine(old_cursor, Some(content)) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let buffer_len = editor.c_viewport().get_buffer_len();
                if cy as usize >= buffer_len {
                    editor.c_mut_viewport().buffer.lines.push(content.clone());
                    editor.cursor.1 += 1;
                } else {
                    editor
                        .c_mut_viewport()
                        .buffer
                        .lines
                        .insert(cy as usize, content.clone());
                }
                editor.c_mut_viewport().top = old_cursor.top;
                editor.cursor.1 = old_cursor.cursor.1;

                // put the line at the center of screen if possible
                editor.buffer_actions.push(Action::CenterLine)
            }
            Action::CenterLine => {
                let mut cursor = editor.cursor.clone();
                editor.c_mut_viewport().center_line(&mut cursor);
                editor.cursor = cursor;
            }
            Action::UndoNewLine(old_cursor) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let c_mut_viewport = editor.c_mut_viewport();
                c_mut_viewport.buffer.remove(cy as usize);
                c_mut_viewport.top = old_cursor.top;
                editor.cursor.1 = old_cursor.cursor.1;
            }
            Action::UndoMultiple(actions) => {
                for action in actions.iter().rev() {
                    action.execute(editor)?;
                }
            }
            Action::ExecuteCommand => {
                let cmd = editor.command.as_str();
                if let Some(action) = Command::execute(cmd) {
                    editor.buffer_actions.push(action);
                }
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }
            Action::RemoveCommandChar => {
                if !editor.command.is_empty() {
                    editor.command.pop();
                }
            }
            Action::EnterFileOrDirectory => {
                let (_, y) = editor.v_cursor();
                if let Some(path) = editor.c_viewport().buffer.get(y as usize) {
                    // editor.viewport.clear_draw(&mut editor.stdout)?;
                    {
                        // DEV_ERROR
                        // let c_mut_viewport = editor.c_mut_viewport();
                        // c_mut_viewport.clear_at(
                        //     &mut editor.stdout,
                        //     c_mut_viewport.min_vwidth,
                        //     c_mut_viewport.min_vheight,
                        //     c_mut_viewport.vwidth,
                        //     c_mut_viewport.vheight,
                        // )?;
                    }
                    editor.reset_cursor();
                    match metadata(&path)?.is_dir() {
                        true => {
                            editor.c_mut_viewport().buffer = Buffer::new(Some(path));
                        }
                        false => {
                            // DEV_ERROR: CHANGER ICI POUR TROUVER LE BON VIEWPORT
                            // editor.buffer_viewport_or_explorer.buffer = Buffer::new(Some(path));
                            editor.buffer_actions.push(Action::SwapViewportToExplorer);
                        }
                    }
                }
            }
            Action::SwapViewportToExplorer => {
                let c_mut_viewport = editor.c_mut_viewport();
                let vwidth = c_mut_viewport.vwidth;
                let vheight = c_mut_viewport.vheight;

                // DEV_ERROR: CHANGER ICI POUR TROUVER LE BON VIEWPORT
                // c_mut_viewport.clear_at(&mut editor.stdout, 0, 0, vwidth, vheight)?;

                editor.reset_cursor();

                // DEV_ERROR: CHANGER ICI POUR TROUVER LE BON VIEWPORT
                // std::mem::swap(
                //     &mut editor.viewport,
                //     &mut editor.buffer_viewport_or_explorer,
                // );
            }

            Action::SwapViewportToPopupExplorer => {
                editor.reset_cursor();

                match editor.c_viewport().is_popup {
                    true => {
                        editor.c_mut_viewport().as_normal();

                        // DEV_ERROR: CHANGER ICI POUR TROUVER LE BON VIEWPORT
                        // std::mem::swap(
                        //     &mut editor.viewport,
                        //     &mut editor.buffer_viewport_or_explorer,
                        // );
                    }
                    false => {
                        // DEV_ERROR: CHANGER ICI POUR TROUVER LE BON VIEWPORT
                        // std::mem::swap(
                        //     &mut editor.viewport,
                        //     &mut editor.buffer_viewport_or_explorer,
                        // );
                        editor.c_mut_viewport().as_popup()
                    }
                }
            }

            _ => {}
        }
        if !editor.buffer_actions.is_empty() {
            if let Some(action) = editor.buffer_actions.pop() {
                action.execute(editor)?;
            }
        }
        Ok(())
    }
}
