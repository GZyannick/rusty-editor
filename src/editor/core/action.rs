use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use crate::editor::{MOVE_PREV_OR_NEXT_LINE, TERMINAL_LINE_LEN_MINUS};
use crate::log_message;

use super::super::Editor;
use super::command::Command;
use super::mode::Mode;

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
    UndoDeleteLine(u16, u16, Option<String>), //cursor.1 , top, content
    UndoNewLine(u16, u16),
    UndoMultiple(Vec<Action>),
    ExecuteCommand,
    RemoveCommandChar,
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
                    && (editor.cursor.1 > 0 || editor.viewport.top > 0)
                    && MOVE_PREV_OR_NEXT_LINE
                {
                    // if we are at the start of the line go ot the prev line if exist
                    // and move the cursor to the end of the line
                    editor.move_prev_line();
                    editor.cursor.0 = editor.viewport.get_line_len(&editor.cursor);
                }
            }

            Action::MoveDown => {
                editor.move_next_line();
            }
            Action::AddChar(c) => {
                let cursor_viewport = editor.v_cursor();
                // editor.undo_insert_actions();
                editor
                    .undo_insert_actions
                    .push(Action::RemoveCharAt(cursor_viewport));

                editor.viewport.buffer.add_char(*c, cursor_viewport);
                editor.cursor.0 += 1;
            }
            Action::RemoveCharAt(cursor) => {
                if editor.viewport.get_line_len(cursor) > 0 {
                    editor.viewport.buffer.remove_char(*cursor);
                }
            }
            Action::RemoveChar => {
                let cursor_viewport = editor.v_cursor();
                match cursor_viewport.0 > 0 {
                    true => {
                        editor.cursor.0 -= 1;
                        editor
                            .viewport
                            .buffer
                            .remove_char((cursor_viewport.0 - 1, cursor_viewport.1));
                    }
                    false if cursor_viewport.1 > 0 => {
                        // we get the size of the prev line before change
                        // because we want the text that will be added behind the cursor
                        let new_x_pos = editor
                            .viewport
                            .get_line_len(&(editor.cursor.0, editor.cursor.1 - 1));
                        editor.viewport.buffer.remove_char_line(cursor_viewport);
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
                editor.viewport.buffer.new_line(v_cursor, false);
                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));
                editor.cursor.0 = 0;

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(editor.cursor.1, editor.viewport.top));
            }
            Action::NewLineInsertionBelowCursor => {
                let (v_x, v_y) = editor.v_cursor();
                editor.viewport.buffer.new_line((v_x, v_y + 1), false);
                editor.move_next_line();
                editor.cursor.0 = 0;

                editor.buffer_actions.push(Action::EnterMode(Mode::Insert));

                editor
                    .undo_actions
                    .push(Action::UndoNewLine(editor.cursor.1, editor.viewport.top));
            }
            Action::NewLine => {
                let (v_x, v_y) = editor.v_cursor();
                editor.viewport.buffer.new_line((v_x, v_y + 1), false);
                editor.cursor.0 = 0;
                editor.move_next_line();
            }
            Action::SaveFile => {
                editor.viewport.buffer.save()?;
            }
            Action::PageUp => {
                editor.viewport.page_up();
            }
            Action::StartOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 = 0;
            }
            Action::EndOfLine => {
                editor.clear_buffer_x_cursor();
                editor.cursor.0 =
                    editor.viewport.get_line_len(&editor.cursor) - TERMINAL_LINE_LEN_MINUS;
            }
            Action::PageDown => {
                editor.viewport.page_down(&editor.cursor);
            }
            Action::WaitingCmd(c) => {
                editor
                    .stdout
                    .queue(cursor::SetCursorStyle::BlinkingUnderScore)?;
                editor.waiting_command = Some(*c);
            }
            Action::DeleteLine => {
                let (_, y) = editor.v_cursor();
                let content = editor.viewport.buffer.get(y as usize).clone();
                editor.viewport.buffer.remove(y as usize);

                editor.undo_actions.push(Action::UndoDeleteLine(
                    editor.cursor.1,
                    editor.viewport.top,
                    content,
                ));
            }
            Action::DeleteWord => editor.viewport.buffer.remove_word(editor.v_cursor()),
            Action::StartOfFile => {
                editor.viewport.move_top();
                editor.cursor.1 = 0;
            }
            Action::EndOfFile => {
                editor.viewport.move_end(&mut editor.cursor);
            }
            Action::Undo => {
                if let Some(action) = editor.undo_actions.pop() {
                    action.execute(editor)?;
                    // TODO: i think this is undo wo need to move the cursor where it was before
                }
            }
            Action::UndoDeleteLine(y, top, Some(content)) => {
                let cy = y + top;
                let buffer_len = editor.viewport.get_buffer_len();
                if cy as usize >= buffer_len {
                    editor.viewport.buffer.lines.push(content.clone());
                    editor.cursor.1 += 1;
                } else {
                    editor
                        .viewport
                        .buffer
                        .lines
                        .insert(cy as usize, content.clone());
                }
                editor.viewport.top = *top;

                // put the line at the center of screen if possible
                editor.viewport.center_line(&mut editor.cursor);
            }
            Action::CenterLine => {
                editor.viewport.center_line(&mut editor.cursor);
            }
            Action::UndoNewLine(y, top) => {
                let cy = y + top;
                editor.viewport.buffer.remove(cy as usize);
                editor.viewport.top = *top;
                editor.cursor.1 = *y;
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
