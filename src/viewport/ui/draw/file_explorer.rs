use std::path::PathBuf;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::viewport::Viewport;

pub fn draw_file_explorer(
    viewport: &Viewport,
    stdout: &mut std::io::Stdout,
) -> anyhow::Result<u16> {
    let mut y = viewport.min_vheight;
    for (i, line) in viewport.buffer.lines.iter().enumerate() {
        viewport.draw_line_number(stdout, y)?;

        let icon = match PathBuf::from(line).is_dir() {
            true => " \u{f115}",
            false => match line.split('.').last() {
                Some("txt") => " \u{f15c}",
                Some("md") => " \u{f48a}",
                Some("rs") => " \u{e7a8}",
                Some("py") => " \u{e73c}",
                Some("png") | Some("jpg") => " \u{f1c5}",
                _ => " \u{f016}",
            },
        };

        // we skip the ../ line
        //            // we skip the ../ line
        let line = match i > 0 {
            true if line.starts_with(viewport.buffer.path.as_str()) => {
                let mut path = viewport.buffer.path.clone();
                if !path.ends_with("/") {
                    path.push('/');
                }
                line.replacen(path.as_str(), "", 1).to_string()
            }
            true => line.to_string(),
            false => line.to_string(),
        };

        let path = format!(" {:<width$} ", line, width = viewport.vwidth as usize - 4);
        stdout
            .queue(cursor::MoveTo(viewport.min_vwidth - 1, y))?
            .queue(PrintStyledContent(
                icon.with(Color::White).on(viewport.bg_color).bold(),
            ))?
            .queue(cursor::MoveTo(viewport.min_vwidth + 1, y))?
            .queue(PrintStyledContent(
                path.with(Color::White).on(viewport.bg_color),
            ))?;
        y += 1;
    }
    Ok(y)
}
