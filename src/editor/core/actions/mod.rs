pub mod action;
pub mod deletion;
pub mod insertion;
pub mod movement;
pub mod search;
pub mod undo;
pub mod viewport;
pub mod yank_past;
use std::fs::metadata;
use std::io::Write;

use action::Action;
use anyhow::Ok;
use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use super::super::Editor;
use super::command::Command;
use super::mode::Mode;
use crate::buff::Buffer;
use crate::editor::ui::clear::ClearDraw;
use crate::editor::ui::modal::{
    create::ModalCreateFD, delete::ModalDeleteFD, rename::ModalRenameFD,
};
use crate::editor::TERMINAL_SIZE_MINUS;
use crate::viewport::Viewport;
use crate::{editor, log_message};

impl ClearDraw for Viewport {}

impl Action {
    // handle insert and leaving visual mode
    fn enter_mode_visual<W: Write>(
        &self,
        editor: &mut Editor<W>,
        mode: &Mode,
    ) -> anyhow::Result<()> {
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
    fn enter_mode_insert<W: Write>(
        &self,
        editor: &mut Editor<W>,
        mode: &Mode,
    ) -> anyhow::Result<()> {
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
    fn enter_mode_command<W: Write>(
        &self,
        editor: &mut Editor<W>,
        mode: &Mode,
    ) -> anyhow::Result<()> {
        // if we leave command clear bottom line
        if matches!(editor.mode, Mode::Command) && !matches!(mode, Mode::Command) {
            editor.command = String::new();
        }
        Ok(())
    }

    pub fn execute<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        // i could use a tree pattern like in movement i call delete in delete i call for find ...
        // but i prefer to call all of them in a single file
        self.movement(editor)?;
        self.deletion(editor)?;
        self.search(editor)?;
        self.insertion(editor)?;
        self.undo(editor)?;
        self.yank_past(editor)?;
        self.viewport(editor)?;

        // other that dont really need a file for themselve
        match self {
            Action::EnterMode(mode) => {
                // to check if the viewport is modifiable to enter the insert_mode
                if matches!(mode, Mode::Insert) && !editor.viewports.c_viewport().modifiable {
                    editor.toast.error("viewport cannot be modifiable".into());
                    return Ok(());
                }
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
                    current_viewport.modifiable = true;
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
                            let viewport = editor.viewports.c_mut_viewport();
                            viewport.modifiable = true;
                            viewport.buffer = Buffer::new(Some(path));
                        }
                        false => {
                            let mut viewport = Viewport::new(
                                Buffer::new(Some(path)),
                                editor.size.0,
                                editor.size.1 - TERMINAL_SIZE_MINUS,
                                0,
                                true,
                            );
                            viewport.buffer.set_query_language(&viewport.languages);
                            editor.viewports.push(viewport);
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

                editor.viewports.is_explorer = !editor.viewports.is_explorer;
                editor.viewports.c_mut_viewport().as_normal();
            }
            Action::SwapViewportToPopupExplorer => {
                editor.reset_cursor();
                editor.viewports.is_explorer = !editor.viewports.is_explorer;
                editor.viewports.c_mut_viewport().as_normal();
                editor.viewports.explorer.as_popup();
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

            Action::HelpKeybinds(keybind_type) => {
                //TODO: For now we need to save all viewports before viewing keybinds because it remove the current
                //buffer
                if !editor.viewports.viewports_save_status()? {
                    return Ok(());
                }
                if let Some(viewport) = editor.viewports.get_original_viewport() {
                    viewport.modifiable = false;
                    viewport.buffer = match keybind_type {
                        Some(keybind_type) => Buffer::new_tmp(
                            editor.keybinds.show_specific_keybinds(keybind_type),
                            "Keybinds".to_string(),
                        ),
                        None => {
                            Buffer::new_tmp(editor.keybinds.show_keybinds(), "Keybinds".to_string())
                        }
                    }
                }
            }
            Action::PushViewport => {
                editor.viewports.push(Viewport::new(
                    Buffer::new(Some("./src/main.rs".to_string())),
                    editor.size.0,
                    editor.size.1,
                    0,
                    true,
                ));
                log_message!("Hello");
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

#[cfg(test)]
mod tests_other_actions {
    use std::fs::{self, File};
    use std::io::Cursor;

    use crate::buff::Buffer;
    use crate::editor::core::actions::action::Action;
    use crate::editor::core::mode::Mode;
    use crate::editor::Editor;

    fn mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::default()
    }

    const TMP_DIR: &str = "./target/tmp";

    // --- EnterMode Tests ---
    #[test]
    fn test_enter_mode_insert_not_modifiable() {
        let mut editor = mock_editor();
        editor.viewports.c_mut_viewport().modifiable = false;

        Action::EnterMode(Mode::Insert)
            .execute(&mut editor)
            .unwrap();

        assert_ne!(editor.mode, Mode::Insert);
        assert_eq!(
            editor.toast._last_message(),
            Some("viewport cannot be modifiable")
        );
    }

    #[test]
    fn test_enter_mode_successfully() {
        let mut editor = mock_editor();
        editor.viewports.c_mut_viewport().modifiable = true;

        Action::EnterMode(Mode::Insert)
            .execute(&mut editor)
            .unwrap();

        assert_eq!(editor.mode, Mode::Insert);
    }

    // --- Save File Test ---
    #[test]
    fn test_save_file() {
        let mut editor = mock_editor();
        editor.viewports.c_mut_viewport().buffer.path = "test_file.txt".to_string();
        File::create("test_file.txt").unwrap();

        Action::Save.execute(&mut editor).unwrap();

        assert!(fs::metadata("test_file.txt").is_ok());
        assert_eq!(
            editor.toast._last_message(),
            Some("file: test_file.txt is saved")
        );

        fs::remove_file("test_file.txt").unwrap();
    }

    // --- File Operations ---
    #[test]
    fn test_create_file_or_directory_success() {
        let mut editor = mock_editor();
        editor.viewports.is_explorer = true;
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(TMP_DIR.to_string()));
        let filename = format!("{TMP_DIR}/test_folder");

        match Action::CreateFileOrDirectory("test_folder/".to_string()).execute(&mut editor) {
            Ok(a) => println!("{a:?}"),
            Err(err) => println!("this is error: {err:?}"),
        }
        assert!(fs::metadata(&filename).is_ok());
        assert_eq!(
            editor.toast._last_message(),
            Some("test_folder/ has been created")
        );
        fs::remove_dir(filename).unwrap();
    }

    #[test]
    fn test_rename_and_delete_file_or_directory() {
        let mut editor = mock_editor();
        editor.viewports.is_explorer = true;
        File::create(format!("{TMP_DIR}/old_file.txt")).unwrap();
        editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(TMP_DIR.to_string()));
        let filename = format!("{TMP_DIR}/new_file.txt");
        editor.cursor.1 = 1; // poiting the file to rename (old_file.txt)
        println!(
            "current_file {:?}",
            editor.viewports.c_viewport().buffer.get(1)
        );

        match Action::RenameFileOrDirectory(filename.to_string()).execute(&mut editor) {
            Ok(_) => (),
            Err(e) => println!("{e}"),
        }

        assert!(fs::metadata(&filename).is_ok());
        println!("toast: {:?}", editor.toast._last_message());

        match Action::DeleteFileOrDirectory.execute(&mut editor) {
            Ok(_) => (),
            Err(e) => println!("err: {e}"),
        }

        assert!(fs::metadata(&filename).is_err());
        if fs::metadata(&filename).is_ok() {
            fs::remove_file(&filename).unwrap();
        }

        assert_eq!(
            editor.toast._last_message(),
            Some("./target/tmp/new_file.txt has been deleted")
        );
    }

    // #[test]
    // fn test_delete_file_or_directory() {
    //     let mut editor = mock_editor();
    //     editor.viewports.set_current_to_file_explorer_viewport();
    //     editor.viewports.c_mut_viewport().buffer = Buffer::new(Some(TMP_DIR.to_string()));
    //     let filename = format!("{TMP_DIR}/delete_me.txt");
    //     File::create(&filename).unwrap();
    //     editor.cursor.1 = 1; // poiting the file to rename (delete_me.txt)
    //
    //         }
    //
    // // --- Modal Tests ---
    // #[test]
    // fn test_leave_modal() {
    //     let mut editor = mock_editor();
    //     editor.modal = Some("Test Modal".to_string());
    //
    //     Action::LeaveModal.execute(&mut editor).unwrap();
    //
    //     assert!(editor.modal.is_none());
    // }
    //
    // #[test]
    // fn test_create_input_modal() {
    //     let mut editor = mock_editor();
    //
    //     Action::CreateInputModal.execute(&mut editor).unwrap();
    //
    //     assert!(editor.modal.is_some());
    // }
    //
    // --- Navigation Tests ---
    #[test]
    fn test_goto_parent_directory() {
        let mut editor = mock_editor();
        let current_path = std::env::current_dir().unwrap();
        let parent_path = current_path.parent().unwrap().to_str().unwrap().to_string();

        editor.viewports.c_mut_viewport().buffer.path = current_path.to_str().unwrap().to_string();

        Action::GotoParentDirectory.execute(&mut editor).unwrap();

        assert_eq!(editor.viewports.c_viewport().buffer.path, parent_path);
    }

    // --- Command Execution Tests ---
    #[test]
    fn test_execute_command() {
        let mut editor = mock_editor();
        editor.command = "w".to_string();

        Action::ExecuteCommand.execute(&mut editor).unwrap();
        assert!(
            editor.toast._last_message().is_some(),
            "Should contains a toast"
        )
    }
}
