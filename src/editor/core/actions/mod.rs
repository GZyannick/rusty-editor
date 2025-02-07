pub mod action;
pub mod deletion;
pub mod insertion;
pub mod movement;
pub mod search;
pub mod undo;
pub mod yank_past;
use std::fs::metadata;

use action::Action;
use anyhow::Ok;
use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use crate::buff::Buffer;
use crate::editor::ui::clear::ClearDraw;
use crate::editor::ui::modal::{
    create::ModalCreateFD, delete::ModalDeleteFD, rename::ModalRenameFD,
};
use crate::viewport::Viewport;

use super::super::Editor;
use super::command::Command;
use super::mode::Mode;

impl ClearDraw for Viewport {}

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
        // i could use a tree pattern like in movement i call delete in delete i call for find ...
        // but i prefer to call all of them in a single file
        self.movement(editor)?;
        self.deletion(editor)?;
        self.search(editor)?;
        self.insertion(editor)?;
        self.undo(editor)?;
        self.yank_past(editor)?;

        // other that dont really need a file for themselve
        match self {
            Action::EnterMode(mode) => {
                self.enter_mode_insert(editor, mode)?;
                self.enter_mode_visual(editor, mode)?;
                self.enter_mode_command(editor, mode)?;
                editor.mode = *mode;
            }
            Action::Save => {
                if !editor.viewports.c_viewport().is_file_explorer() {
                    let current_viewport = editor.viewports.c_mut_viewport();
                    current_viewport.buffer.save()?;
                    editor
                        .toast
                        .indication(format!("file: {} is saved", current_viewport.buffer.path));
                }
            }
            Action::CreateFileOrDirectory(filename) => {
                let current_viewport = editor.viewports.c_mut_viewport();
                let is_created = current_viewport
                    .buffer
                    .create_files_or_directories(filename)?;
                editor.buffer_actions.push(Action::LeaveModal);
                match is_created {
                    true => {
                        editor
                            .toast
                            .indication(format!("{filename} has been created"));
                    }
                    false => {
                        editor
                            .toast
                            .error(format!("error: couldnt create {filename}"));
                    }
                }
            }
            Action::RenameFileOrDirectory(filename) => {
                let y = editor.v_cursor().1 as usize;
                let current_viewport = editor.viewports.c_mut_viewport();
                if let Some(file) = current_viewport.buffer.lines.get_mut(y) {
                    std::fs::rename(file.clone(), filename)?;
                    *file = filename.clone();
                    editor
                        .toast
                        .indication(format!("successfull rename too {filename}"));
                    editor.buffer_actions.push(Action::LeaveModal);
                }
            }
            Action::DeleteFileOrDirectory => {
                let y = editor.v_cursor().1 as usize;
                let current_viewport = editor.viewports.c_mut_viewport();
                if let Some(path) = current_viewport.buffer.lines.get_mut(y) {
                    match std::fs::metadata(&path) {
                        std::io::Result::Ok(meta) => {
                            match meta.is_file() {
                                true => {
                                    std::fs::remove_file(&path)?;
                                }
                                false => {
                                    std::fs::remove_dir(&path)?;
                                }
                            }
                            editor.toast.indication(format!("{path} has been deleted"));
                            current_viewport.buffer.remove(y);
                        }
                        std::io::Result::Err(_) => {
                            editor.toast.error(format!("Couldnt Delete {path}"));
                        }
                    };
                    editor.buffer_actions.push(Action::LeaveModal);
                }
            }
            Action::WaitingCmd(c) => {
                editor
                    .stdout
                    .queue(cursor::SetCursorStyle::BlinkingUnderScore)?;
                editor.waiting_command = Some(*c);
            }
            Action::ExecuteCommand => {
                let cmd = editor.command.as_str();
                if let Some(action) = Command::execute(cmd) {
                    editor.buffer_actions.push(action);
                }
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }
            // if it enter this Action::Quit its because file arent save so we need to leave the
            // Mode::Command
            Action::Quit => {
                editor.toast.error(format!(
                    "file: {} is not saved",
                    editor.viewports.c_viewport().buffer.path
                ));
                editor.buffer_actions.push(Action::EnterMode(Mode::Normal));
            }
            Action::GotoParentDirectory => {
                let current_viewport = editor.viewports.c_mut_viewport();
                if let Some(parent_buffer) = current_viewport.buffer.parent_dir() {
                    current_viewport.buffer = parent_buffer;
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
                            editor.buffer_actions.push(Action::GotoParentDirectory);
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
            Action::LeaveModal => {
                editor.modal = None;
            }
            Action::CreateInputModal => {
                let modal_input =
                    ModalCreateFD::new("Enter The name of dir (dir finish with /) ".into());
                editor.set_modal(Box::new(modal_input));
            }
            Action::RenameInputModal => {
                let current_viewport = editor.viewports.c_viewport();
                if let Some(line) = current_viewport.buffer.get(editor.v_cursor().1 as usize) {
                    let modal_input =
                        ModalRenameFD::new(format!("Enter the name for {line}"), line.clone());
                    editor.set_modal(Box::new(modal_input));
                }
            }
            Action::DeleteInputModal => {
                let current_viewport = editor.viewports.c_viewport();
                if let Some(line) = current_viewport.buffer.get(editor.v_cursor().1 as usize) {
                    let line = line.replace(&current_viewport.buffer.path, "");
                    let modal_input =
                        ModalDeleteFD::new(format!("Are you you wan to delete {line} Y/N"));
                    editor.set_modal(Box::new(modal_input));
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
