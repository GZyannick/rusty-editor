use std::io::Write;

use crate::editor::Editor;
use anyhow::{Ok, Result};
use crossterm::QueueableCommand;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
};

use super::actions::action::Action;
use super::mode::Mode;

impl<W: Write> Editor<W> {
    pub fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            let modifiers = ev.modifiers;

            if let Some(ref mut modal) = self.modal {
                // this keybinds will still be related to handle_action
                // and not keybind manager because there are not global to the editor
                // but specific to each modal
                return modal.handle_action(&code, &modifiers);
            }

            if let Some(c) = self.waiting_command {
                let action = self.handle_waiting_command(c, &code);
                self.waiting_command = None;
                self.stdout
                    .queue(cursor::SetCursorStyle::DefaultUserShape)?;
                return Ok(action);
            }

            let nav = self.navigation(&code, &modifiers)?;
            if nav.is_some() {
                return Ok(nav);
            }

            return self.handle_keybinds(code, modifiers);
        }
        Ok(None)
    }

    fn handle_keybinds(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let is_file_explorer = self.viewports.c_viewport().is_file_explorer();
        let result = self.keybinds.handle_keybinds(
            self.mode,
            code,
            modifiers,
            &self.v_cursor(),
            &self.command,
            is_file_explorer,
        );

        // let mut temp_keybinds = self.keybinds.take_by_mode(&self.mode, is_file_explorer);
        // let result = KeybindManager::handle_keybind(&mut temp_keybinds, code, modifiers, self);
        // self.keybinds
        //     .push_by_mode(&self.mode, temp_keybinds, is_file_explorer);
        Ok(result)
    }

    fn handle_waiting_command(&mut self, c: char, code: &KeyCode) -> Option<Action> {
        match c {
            'd' => match code {
                KeyCode::Char('d') => Some(Action::DeleteLine),
                KeyCode::Char('w') => Some(Action::DeleteWord),
                _ => None,
            },
            'g' => match code {
                KeyCode::Char('g') => Some(Action::StartOfFile),
                _ => None,
            },
            'z' => match code {
                KeyCode::Char('z') => Some(Action::CenterLine),
                _ => None,
            },
            'y' => match code {
                KeyCode::Char('y') => Some(Action::YankLine),
                _ => None,
            },
            ' ' => match code {
                KeyCode::Char('e') => Some(Action::SwapViewportToPopupExplorer),
                KeyCode::Char('-') => Some(Action::SwapViewportToExplorer),
                _ => None,
            },
            _ => None,
        }
    }

    fn navigation(&mut self, code: &KeyCode, modifiers: &KeyModifiers) -> Result<Option<Action>> {
        let mut action: Option<Action> = None;

        if matches!(self.mode, Mode::Command) || matches!(self.mode, Mode::Search) {
            return Ok(action);
        }

        action = match code {
            KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Left => Some(Action::MoveLeft),
            KeyCode::Right => Some(Action::MoveRight),
            _ => None,
        };

        if !matches!(self.mode, Mode::Insert)
            && action.is_none()
            && matches!(modifiers, &KeyModifiers::NONE)
        {
            action = match code {
                KeyCode::Char('h') => Some(Action::MoveLeft),
                KeyCode::Char('j') => Some(Action::MoveDown),
                KeyCode::Char('k') => Some(Action::MoveUp),
                KeyCode::Char('l') => Some(Action::MoveRight),
                KeyCode::Char('w') => Some(Action::MoveNext), // Move next until the char is not
                // the same type than before
                KeyCode::Char('b') => Some(Action::MovePrev), // Move prev until the char is not
                // the same type than before
                // exemple if char is a letter Move until char is diff from letter
                _ => None,
            }
        };

        Ok(action)
    }
}

#[cfg(test)]
mod tests_editor_handler {
    use super::*;
    use crossterm::event::{self, KeyEventState};
    use std::io::Cursor;

    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::default()
    }

    #[test]
    fn test_handle_action_key_navigation() -> Result<()> {
        let mut editor = create_mock_editor();
        // Simulate a KeyEvent for 'Arrow Down'
        let key_event = event::Event::Key(event::KeyEvent {
            kind: event::KeyEventKind::Press,
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            state: KeyEventState::NONE,
        });

        let action = editor.handle_action(key_event)?;

        // Check that the action is MoveDown
        assert_eq!(action, Some(Action::MoveDown));
        Ok(())
    }

    #[test]
    fn test_handle_navigation_with_letter() -> Result<()> {
        let mut editor = create_mock_editor();

        let move_left_event = event::Event::Key(event::KeyEvent {
            kind: event::KeyEventKind::Press,
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            state: KeyEventState::NONE,
        });

        let action = editor.handle_action(move_left_event)?;

        // Check that the action is MoveLeft
        assert_eq!(action, Some(Action::MoveLeft));
        Ok(())
    }

    #[test]
    fn test_handle_action_waiting_command() -> Result<()> {
        let mut editor = create_mock_editor();
        editor.waiting_command = Some('d'); // Set the waiting command to 'd'

        // Simulate the KeyEvent for 'd' key press
        let key_event = event::Event::Key(event::KeyEvent {
            kind: event::KeyEventKind::Press,
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            state: KeyEventState::NONE,
        });

        // Expect action to be DeleteLine as per 'd' waiting command
        let action = editor.handle_action(key_event)?;

        assert_eq!(action, Some(Action::DeleteLine));
        Ok(())
    }

    #[test]
    fn test_handle_action_invalid_key() -> Result<()> {
        let mut editor = create_mock_editor();

        // Simulate a key event that shouldn't trigger any action (e.g., a random key)
        let key_event = event::Event::Key(event::KeyEvent {
            kind: event::KeyEventKind::Press,
            code: KeyCode::Char('+'),
            modifiers: KeyModifiers::NONE,
            state: KeyEventState::NONE,
        });

        // The action should be None, as 'x' doesn't trigger any valid action.
        let action = editor.handle_action(key_event)?;

        assert_eq!(action, None);
        Ok(())
    }

    #[test]
    fn test_handle_action_with_release() -> Result<()> {
        let mut editor = create_mock_editor();
        // Simulate a KeyEvent for 'Arrow Up' key release
        let key_event = event::Event::Key(event::KeyEvent {
            kind: event::KeyEventKind::Release,
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            state: KeyEventState::NONE,
        });

        // The action should be None, as it's a key release
        let action = editor.handle_action(key_event)?;
        assert_eq!(action, None);
        Ok(())
    }
}
