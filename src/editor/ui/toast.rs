use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

#[derive(Debug)]
pub struct ToastMessage {
    message: String,
    start_time: Option<Instant>,
    is_error: bool,
}

impl ToastMessage {
    pub fn is_elapsed(&mut self) -> bool {
        match self.start_time {
            Some(instant) => {
                if instant.elapsed() >= Duration::new(2, 0) {
                    return true;
                }
            }
            None => {
                self.start_time = Some(Instant::now());
            }
        }
        false
    }
}
#[derive(Debug)]
pub struct Toast {
    messages: Vec<ToastMessage>,
}

impl Toast {
    pub fn new() -> Self {
        Self { messages: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn error(&mut self, err: String) {
        self.messages.push(ToastMessage {
            message: err,
            is_error: true,
            start_time: None,
        });
    }

    pub fn indication(&mut self, msg: String) {
        self.messages.push(ToastMessage {
            message: msg,
            is_error: false,
            start_time: None,
        });
    }

    pub fn _last_message(&self) -> Option<&str> {
        if let Some(toast_message) = self.messages.last() {
            return Some(&toast_message.message);
        }
        None
    }

    pub fn draw<W: std::io::Write>(&mut self, stdout: &mut W, size_x: &u16) -> anyhow::Result<()> {
        let messages = &mut self.messages;
        if let Some(toast_message) = messages.first_mut() {
            if toast_message.is_elapsed() {
                self.messages.remove(0);
                return Ok(());
            }

            let message = &toast_message.message;
            let start_block = message.len() as u16 + 5;
            let start_x = size_x.saturating_sub(start_block + 1);
            let color = match toast_message.is_error {
                true => Color::Red,
                false => Color::Green,
            };
            stdout
                .queue(cursor::MoveTo(start_x, 1))?
                .queue(PrintStyledContent(
                    format!("╭{}╮", "─".repeat(start_block as usize - 3)).with(color),
                ))?
                .queue(cursor::MoveTo(start_x, 2))?
                .queue(PrintStyledContent(
                    format!("│ {message} │").with(color).bold(),
                ))?
                .queue(cursor::MoveTo(start_x, 3))?
                .queue(PrintStyledContent(
                    format!("╰{}╯", "─".repeat(start_block as usize - 3)).with(color),
                ))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_toast {
    use std::io::Cursor;

    use crate::editor::Editor;

    // Helper function to create a mock Editor with Cursor<Vec<u8>>
    fn create_mock_editor() -> Editor<Cursor<Vec<u8>>> {
        Editor::<Cursor<Vec<u8>>>::default()
    }

    #[test]
    fn test_toast_indication() {
        let mut editor = create_mock_editor();
        editor.toast.indication("Hello_word".to_string());

        let first_toast = editor.toast.messages.first().unwrap();

        assert!(!first_toast.is_error, "should not be an error");
        assert!(
            first_toast.message.eq("Hello_word"),
            "the content should be Hello_word"
        )
    }
}
