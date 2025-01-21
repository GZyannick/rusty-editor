use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

#[derive(Debug)]
pub struct Toast {
    errors: Vec<String>,
    _indications: Vec<String>,
    // could add other thing like indication ( file is saved)
}

impl Toast {
    pub fn new() -> Self {
        Self {
            errors: vec![],
            _indications: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self._indications.is_empty()
    }

    pub fn error(&mut self, err: &str) {
        self.errors.push(err.to_string());
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout, size_x: &u16) -> anyhow::Result<()> {
        if let Some(error) = self.errors.first() {
            let start_block = error.len() as u16 + 5;
            let start_x = size_x.saturating_sub(start_block + 1);
            stdout
                .queue(cursor::MoveTo(start_x, 1))?
                .queue(PrintStyledContent(
                    format!("╭{}╮", "─".repeat(start_block as usize - 3)).with(Color::Red),
                ))?
                .queue(cursor::MoveTo(start_x, 2))?
                .queue(PrintStyledContent(
                    format!("│ {error} │").with(Color::Red).bold(),
                ))?
                .queue(cursor::MoveTo(start_x, 3))?
                .queue(PrintStyledContent(
                    format!("╰{}╯", "─".repeat(start_block as usize - 3)).with(Color::Red),
                ))?;
        }
        Ok(())
    }
}
