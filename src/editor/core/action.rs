use std::fs::metadata;

use anyhow::Ok;
use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use crate::buff::Buffer;
use crate::editor::ui::clear::ClearDraw;
use crate::editor::{CursorBlock, TERMINAL_LINE_LEN_MINUS};
use crate::viewport::Viewport;

use super::super::Editor;
use super::chartype::CharType;
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
    UndoDeleteBlock(OldCursorPosition, Vec<Option<String>>), //cursor.1 , top, content
    UndoNewLine(OldCursorPosition),
    UndoMultiple(Vec<Action>),
    UndoCharAt(OldCursorPosition, (u16, u16)),
    ExecuteCommand,
    RemoveCharFrom(bool),
    EnterFileOrDirectory,
    SwapViewportToExplorer,
    SwapViewportToPopupExplorer,
    DeleteBlock,
    YankBlock,
    Past,
    UndoPast(CursorBlock, u16),
    YankLine,
    MovePrev,
    MoveNext,
    ClearToNormalMode,
    AddSearchChar(char),
    FindSearchValue,
    GotoPos((u16, u16)),
    IterNextSearch,
}

impl Action {
    // handle insert and leaving visual mode
    fn enter_mode_visual(&self, editor: &mut Editor, mode: &Mode) -> anyhow::Result<()> {
        // create visual_cursor if we enter Visual Mode
        if !matches!(editor.mode, Mode::Visual) && matches!(mode, Mode::Visual) {
            editor.visual_cursor = Some(editor.cursor);
        }

        // remove visual_cursor if we leave Visual Mode
        if matches!(editor.mode, Mode::Visual) && !matches!(mode, Mode::Visual) {
            editor.visual_cursor = None;
        }
        Ok(())
    }

    // handle insert and leaving insert mode
    fn enter_mode_insert(&self, editor: &mut Editor, mode: &Mode) -> anyhow::Result<()> {
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
        Ok(())
    }

    // handle insert and leaving command mode
    fn enter_mode_command(&self, editor: &mut Editor, mode: &Mode) -> anyhow::Result<()> {
        // if we leave command clear bottom line
        if matches!(editor.mode, Mode::Command) && !matches!(mode, Mode::Command) {
            editor.command = String::new();
        }
        Ok(())
    }

    pub fn execute(&self, editor: &mut Editor) -> anyhow::Result<()> {
        match self {
            Action::Quit => (),
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
            Action::RemoveCharAt(cursor) => {
                if editor.viewports.c_viewport().get_line_len(cursor) > 0 {
                    editor
                        .viewports
                        .c_mut_viewport()
                        .buffer
                        .remove_char(*cursor);
                }
            }
            Action::UndoCharAt(old_cursor, v_cursor) => {
                editor.buffer_actions.push(Action::RemoveCharAt(*v_cursor));
                editor.cursor = old_cursor.cursor;
            }

            Action::RemoveChar => {
                let cursor_viewport = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();
                match cursor_viewport.0 > 0 {
                    true => {
                        editor.cursor.0 -= 1;
                        current_viewport
                            .buffer
                            .remove_char((cursor_viewport.0 - 1, cursor_viewport.1));
                    }
                    false if cursor_viewport.1 > 0 => {
                        // we get the size of the prev line before change
                        // because we want the text that will be added behind the cursor
                        let new_x_pos =
                            current_viewport.get_line_len(&(editor.cursor.0, editor.cursor.1 - 1));
                        current_viewport.buffer.remove_char_line(cursor_viewport);
                        editor.move_prev_line();
                        editor.cursor.0 = new_x_pos;
                    }
                    _ => {}
                }
            }
            Action::EnterMode(mode) => {
                self.enter_mode_insert(editor, mode)?;
                self.enter_mode_visual(editor, mode)?;
                self.enter_mode_command(editor, mode)?;

                editor.mode = *mode;
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
                    .new_line((v_x, v_y + 1), false);
                editor.cursor.0 = 0;
                editor.move_next_line();
            }

            Action::SaveFile => editor.viewports.c_mut_viewport().buffer.save()?,

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

            Action::WaitingCmd(c) => {
                editor
                    .stdout
                    .queue(cursor::SetCursorStyle::BlinkingUnderScore)?;
                editor.waiting_command = Some(*c);
            }

            Action::DeleteLine => {
                let (_, y) = editor.v_cursor();
                let current_viewport = editor.viewports.c_mut_viewport();
                let content = current_viewport.buffer.get(y as usize).clone();
                current_viewport.buffer.remove(y as usize);

                editor.yank_buffer = vec![content.clone()];

                editor.undo_actions.push(Action::UndoDeleteLine(
                    OldCursorPosition::new(editor.cursor, current_viewport.top),
                    content,
                ));
            }

            Action::DeleteWord => {
                let v_cursor = editor.v_cursor();
                editor
                    .viewports
                    .c_mut_viewport()
                    .buffer
                    .remove_word(v_cursor)
            }

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

            Action::CenterLine => {
                editor
                    .viewports
                    .c_mut_viewport()
                    .center_line(&mut editor.cursor);
            }

            Action::UndoNewLine(old_cursor) => {
                let cy = old_cursor.cursor.1 + old_cursor.top;
                let c_mut_viewport = editor.viewports.c_mut_viewport();
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

            Action::RemoveCharFrom(is_search) => {
                let content = match is_search {
                    true => &mut editor.search,
                    false => &mut editor.command,
                };

                match content.is_empty() {
                    true => {
                        editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
                    }
                    false => {
                        content.pop();
                        if *is_search {
                            editor.buffer_actions.push(Action::FindSearchValue)
                        }
                    }
                }
            }

            Action::EnterFileOrDirectory => {
                let (_, y) = editor.v_cursor();
                if let Some(path) = editor.viewports.c_viewport().buffer.get(y as usize) {
                    editor.reset_cursor();
                    // if this is a directory we only change the content of it to the new dir
                    // if its a file we swap to the viewport of file
                    match metadata(&path)?.is_dir() {
                        true if path.eq("../") => {
                            let current_viewport = editor.viewports.c_mut_viewport();
                            if let Some(parent_buffer) = current_viewport.buffer.parent_dir() {
                                current_viewport.buffer = parent_buffer;
                            }
                        }
                        true => {
                            editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(path));
                        }
                        false => {
                            // editor.viewports.c_mut_viewport().as_normal();
                            editor.viewports.get_original_viewport().unwrap().buffer =
                                Buffer::new(Some(path));
                            editor.buffer_actions.push(Action::SwapViewportToExplorer);
                        }
                    }
                }
            }

            Action::SwapViewportToExplorer => {
                let c_mut_viewport = editor.viewports.c_mut_viewport();
                let vwidth = c_mut_viewport.vwidth;
                let vheight = c_mut_viewport.vheight;

                c_mut_viewport.clear_at(&mut editor.stdout, 0, 0, vwidth, vheight)?;

                editor.reset_cursor();

                match editor.viewports.c_viewport().is_file_explorer() {
                    true => editor.viewports.set_current_to_original_viewport(),
                    false => editor.viewports.set_current_to_file_explorer_viewport(),
                }
                editor.viewports.c_mut_viewport().as_normal();
            }

            Action::SwapViewportToPopupExplorer => {
                editor.reset_cursor();
                let c_mut_viewport = editor.viewports.c_mut_viewport();
                match c_mut_viewport.is_file_explorer() {
                    // if this is the file_explorer return to the viewport and make file_explorer
                    // normal again
                    true => {
                        c_mut_viewport.as_normal();
                        editor.viewports.set_current_to_original_viewport();
                    }
                    false => {
                        editor.viewports.set_current_to_file_explorer_viewport();
                        editor.viewports.c_mut_viewport().as_popup();
                    }
                }
            }

            Action::DeleteBlock => {
                if let Some(v_block) = editor.get_visual_block_pos() {
                    let c_mut_viewport = editor.viewports.c_mut_viewport();
                    let v_cursor_start = c_mut_viewport.viewport_cursor(&v_block.start);
                    let v_cursor_end = c_mut_viewport.viewport_cursor(&v_block.end);

                    let block_content: Vec<Option<String>> =
                        c_mut_viewport
                            .buffer
                            .remove_block(v_cursor_start, v_cursor_end, false);

                    // TODO ADD block content to editor.yank_buffer too

                    editor.cursor = v_block.start;
                    editor.undo_actions.push(Action::UndoDeleteBlock(
                        OldCursorPosition::new(v_block.start, c_mut_viewport.top),
                        block_content,
                    ));
                    editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
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
            Action::UndoPast(cursor, top) => {
                let current_viewport = editor.viewports.c_mut_viewport();
                let start_y = cursor.start.1 + top;
                let end_y = cursor.end.1 + top;
                current_viewport.buffer.remove_block(
                    (cursor.start.0, start_y),
                    (cursor.end.0, end_y),
                    true,
                );

                current_viewport.top = *top;
                editor.cursor.1 = cursor.start.1;
                editor.buffer_actions.push(Action::CenterLine);
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
                        editor.cursor.1 += 1;
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
                            editor.cursor.1 -= 1;
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
                                editor.cursor.1 -= 1;
                            }
                        }
                    }
                }
            }

            // allow us to clear search string
            Action::ClearToNormalMode => {
                let current_viewport = editor.viewports.c_mut_viewport();
                current_viewport.clear_search();
                editor.search = String::new();
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }

            // research correspondng value in file when editor.search got updated
            Action::FindSearchValue => {
                let current_viewport = editor.viewports.c_mut_viewport();
                current_viewport.find_occurence(&editor.search);

                if let Some(cursor) = current_viewport.search_pos.first() {
                    editor.buffer_actions.push(Action::GotoPos(*cursor))
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

            Action::IterNextSearch => {
                // iter through the list of search
                let current_viewport = editor.viewports.c_mut_viewport();
                match current_viewport.search_index < current_viewport.search_pos.len() {
                    true => current_viewport.search_index += 1,
                    false => current_viewport.search_index = 0,
                }

                if let Some(cursor) = current_viewport
                    .search_pos
                    .get(current_viewport.search_index)
                {
                    editor.buffer_actions.push(Action::GotoPos(*cursor));
                }
            }
            _ => {}
        }

        // allow us to buffer other actions in action and execute them at the end
        if !editor.buffer_actions.is_empty() {
            if let Some(action) = editor.buffer_actions.pop() {
                action.execute(editor)?;
            }
        }
        Ok(())
    }
}
