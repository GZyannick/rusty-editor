use std::io::Write;

use crate::editor::Editor;
use anyhow::{Ok, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use super::actions::action::Action;

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

            // if let Some(c) = self.waiting_command {
            //     let action = self.handle_waiting_command(c, &code);
            //     self.waiting_command = None;
            //     self.stdout
            //         .queue(cursor::SetCursorStyle::DefaultUserShape)?;
            //     return Ok(action);
            // }

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

        Ok(result)
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
