use anyhow::Result;
use std::{
    io::{Stdout, Write},
    ops::SubAssign,
};

use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    terminal, ExecutableCommand, QueueableCommand,
};

enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    Quit,
}

enum Mode {
    Normal,
    Insert,
    Command,
}

fn handle_action(event: Event) -> Result<Option<Action>> {
    if let event::Event::Key(ev) = event {
        if ev.kind == event::KeyEventKind::Release {
            return Ok(None);
        }
        return match ev.code {
            KeyCode::Char('q') => return Ok(Some(Action::Quit)),
            KeyCode::Char('j') | KeyCode::Down =>  Ok(Some(Action::MoveDown)),
            KeyCode::Char('k') | KeyCode::Up => Ok(Some(Action::MoveUp)),
            KeyCode::Char('h') | KeyCode::Left => Ok(Some(Action::MoveLeft)),
            KeyCode::Char('l') | KeyCode::Right =>Ok(Some(Action::MoveRight)),
            KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
            _ => Ok(None),
        }

    }
    Ok(None)
}

fn main() -> Result<()> {
    let mut cx = 0;
    let mut cy = 0;
    let current_mode = Mode::Normal;
    let mut stdout = std::io::stdout();
    enter_editor(&mut stdout)?;

    loop {
        stdout.queue(cursor::MoveTo(cx, cy))?;
        stdout.flush()?;

        let event = read()?;
        if let Some(action) = handle_action(event)? {
            match action {
                Action::Quit => break,
                Action::MoveUp => if cy > 0{
                    cy.sub_assign(1);
                }
                Action::MoveLeft if cx > 0 => {
                    cx.sub_assign(1);
                }
                Action::MoveRight => {
                    cx += 1;
                }
                Action::MoveDown => {
                    cy += 1;
                }
                _ => {}
            }
        }
    }

    leave_editor(&mut stdout)?;

    Ok(())
}

fn enter_editor(stdout: &mut Stdout) -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

fn leave_editor(stdout: &mut Stdout) -> anyhow::Result<()> {
    stdout.execute(terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
