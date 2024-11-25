use anyhow::{Ok, Result};
use std::{
    fmt::Display,
    io::{Stdout, Write},
    ops::SubAssign,
};

use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{self, size},
    ExecutableCommand, QueueableCommand,
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

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Command => write!(f, "COMMAND"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Normal => write!(f, "NORMAL"),
        }
    }
}

fn handle_action(event: Event, mode: &Mode) -> Result<Option<Action>> {
    if let event::Event::Key(ev) = event {
        if ev.kind == event::KeyEventKind::Release {
            return Ok(None);
        }
        let code: KeyCode = ev.code;

        if let Some(action) = handle_navigation(&code, mode)? {
            return Ok(Some(action)); 
        }

        return match code{
            KeyCode::Char('q') if matches!(mode, Mode::Command) => Ok(Some(Action::Quit)),
            KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
            KeyCode::Char(':') => Ok(Some(Action::EnterMode(Mode::Command))),
            KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
            KeyCode::Char(c) if matches!(mode, Mode::Insert) => Ok(Some(Action::AddChar(c))),
            _ => Ok(None),
        };
    }
    Ok(None)
}

fn handle_navigation(code: &KeyCode, mode: &Mode) -> Result<Option<Action>> {
    match code {
        KeyCode::Down => Ok(Some(Action::MoveDown)),
        KeyCode::Up => Ok(Some(Action::MoveUp)),
        KeyCode::Left => Ok(Some(Action::MoveLeft)),
        KeyCode::Right => Ok(Some(Action::MoveRight)),
        KeyCode::Char('h') if !matches!(mode, Mode::Insert) => Ok(Some(Action::MoveLeft)),
        KeyCode::Char('j') if !matches!(mode, Mode::Insert) => Ok(Some(Action::MoveDown)),
        KeyCode::Char('k') if !matches!(mode, Mode::Insert) => Ok(Some(Action::MoveUp)),
        KeyCode::Char('l') if !matches!(mode, Mode::Insert) => Ok(Some(Action::MoveRight)),
        _ => Ok(None)
    }
}

fn main() -> Result<()> {
    let mut cx = 0;
    let mut cy = 0;
    let mut current_mode = Mode::Normal;
    let mut stdout = std::io::stdout();

    let size = size()?;
    enter_editor(&mut stdout, &size)?;

    loop {
        print_bottom_line(&current_mode, &mut stdout, &size.1)?;
        stdout.queue(cursor::MoveTo(cx, cy))?;
        stdout.flush()?;

        let event = read()?;
        if let Some(action) = handle_action(event, &current_mode)? {
            match action {
                Action::Quit => break,
                Action::MoveUp => {
                    if cy > 0 {
                        cy.sub_assign(1);
                    }
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
                Action::AddChar(c) => {
                    stdout.queue(Print(c.to_string()))?;
                    cx += 1;
                }
                Action::EnterMode(mode) => {
                    current_mode = mode;
                }
                _ => {}
            }
        }
    }

    leave_editor(&mut stdout)?;

    Ok(())
}

fn enter_editor(stdout: &mut Stdout, size: &(u16, u16)) -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;
    stdout.execute(terminal::SetSize(size.0, size.1 - 2))?;
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    Ok(())
}

fn leave_editor(stdout: &mut Stdout) -> anyhow::Result<()> {
    stdout.execute(terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

/// clear the bottom line and print the mode we currently are
fn print_bottom_line(mode: &Mode, stdout: &mut Stdout, sy: &u16) -> anyhow::Result<()> {
    stdout.queue(cursor::MoveTo(0, *sy - 2))?;
    stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
    let mode_style = PrintStyledContent(
        format!(" {mode} ")
            .with(Color::White)
            .on(Color::Rgb {
                r: 188,
                g: 150,
                b: 230,
            })
            .attribute(Attribute::Bold),
    );
    stdout.queue(mode_style)?;
    stdout.flush()?;
    Ok(())
}
