use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    usize,
};

use crate::log_message;

#[derive(Debug)]
pub struct Buffer {
    pub file: Option<File>,
    pub path: String,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn new(file_path: Option<String>) -> Buffer {
        if let Some(f_path) = file_path {
            match std::fs::metadata(f_path.clone()) {
                Ok(metadata) if metadata.is_file() => {
                    return Buffer::from_file(&f_path);
                }
                Ok(metadata) if metadata.is_dir() => {
                    return Buffer::from_dir(&f_path);
                }
                _ => (),
            }
        }

        Buffer {
            file: None,
            lines: vec![],
            path: "Empty".to_string(),
        }
    }

    pub fn get_line(&self, n: usize) -> Option<String> {
        self.lines.get(n).cloned()
    }

    pub fn new_line(&mut self, cursor: (u16, u16), is_take_text: bool) {
        let y_pos: usize = cursor.1 as usize + 1;
        let mut new_line = String::new();

        if is_take_text {
            // slice the part of the string from cursor into the end;
            if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
                let x = cursor.0 as usize;
                let clone_line = line.clone();
                let next_line_content = &clone_line[x..];
                line.replace_range(x.., "");
                new_line.push_str(next_line_content);
            }
        }

        match y_pos > self.lines.len() {
            true => {
                self.lines.push(new_line);
            }
            false => {
                self.lines.insert(y_pos, new_line);
            }
        }
    }

    pub fn add_char(&mut self, c: char, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
            line.insert(cursor.0 as usize - 1_usize, c);
        }
    }

    pub fn remove_char(&mut self, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
            line.remove(cursor.0 as usize - 1_usize);
        }
    }

    pub fn remove_char_line(&mut self, cursor: (u16, u16)) {
        let mut buf = String::new();
        if let Some(line) = self.get_line(cursor.1 as usize) {
            buf = line.clone();
            self.lines.remove(cursor.1 as usize);
        }
        if let Some(prev_line) = self.lines.get_mut(cursor.1 as usize - 1) {
            prev_line.push_str(buf.as_str());
        }
    }

    fn from_file(f_path: &str) -> Buffer {
        let mut file = None;
        let mut lines: Vec<String> = Vec::new();
        let mut path = String::from("Empty");

        if let Ok(mut c_file) = File::open(f_path) {
            let mut buf = String::new();
            c_file.read_to_string(&mut buf).unwrap();
            file = Some(c_file);
            lines = buf.lines().map(|s| s.to_string()).collect();
            path = f_path.to_string();
        }

        Buffer { file, lines, path }
    }

    fn from_dir(_f_path: &str) -> Buffer {
        Buffer {
            file: None,
            lines: vec!["".to_string()],
            path: "Empty".to_string(),
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if let Some(_c_file) = &self.file {
            let mut open_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(self.path.clone())?;
            for line in self.lines.iter() {
                writeln!(open_file, "{line}")?;
            }
        }

        Ok(())
    }
}
