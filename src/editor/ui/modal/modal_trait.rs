use std::{fmt::Debug, io::Write};

use crossterm::{
    cursor,
    event::{KeyCode, KeyModifiers},
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::{
    editor::{core::actions::action::Action, Editor},
    theme::colors,
};

pub trait ModalContent<W: Write>: Debug {
    fn title(&self) -> &str;
    fn body(&self) -> &str;
    fn handle_action(
        &self,
        code: &KeyCode,
        modifiers: &KeyModifiers,
    ) -> anyhow::Result<Option<Action>>;
    fn push(&mut self, ch: char);
    fn pop(&mut self);
    fn draw_modal(&self, editor: &mut Editor<W>) -> anyhow::Result<()>;

    fn draw_default(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        let width = editor.size.0;
        let height = editor.size.1;
        let modal_width = width / 4;
        let modal_height = height / 4;

        let start_x = (width - modal_width) / 2;
        let start_y = (height - modal_height) / 2;

        let title = format!(" {:<width$}", self.title(), width = modal_width as usize);
        editor.stdout.queue(cursor::MoveTo(start_x, start_y))?;
        editor.stdout.queue(PrintStyledContent(
            title.bold().on(Color::from(colors::FADED_PURPLE)),
        ))?;

        let body = format!(" {:<width$}", self.body(), width = modal_width as usize);
        editor.stdout.queue(cursor::MoveTo(start_x, start_y + 1))?;
        editor
            .stdout
            .queue(PrintStyledContent(body.on(Color::from(colors::DARK0_SOFT))))?;

        editor.stdout.flush()?;
        Ok(())
    }
    fn basic_action(&self, code: &KeyCode) -> Option<Action> {
        // with calling basic action in handle action at the end with action.is_some
        // we can override some basic_action
        match code {
            KeyCode::Esc => Some(Action::LeaveModal),
            KeyCode::Backspace => Some(Action::RemoveModalChar),
            KeyCode::Char(c) => Some(Action::AddModalChar(*c)),
            _ => None,
        }
    }
}
