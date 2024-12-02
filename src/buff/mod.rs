use std::{
    clone,
    fs::{File, OpenOptions},
    io::{Read, Write},
    usize,
};

use crate::log_message;

// #[derive(Debug)]
// enum WordType {
//     AlphaNumeric,
//     WhiteSpace,
//     Punctuation,
// }
//
// impl WordType {
//     pub fn get_type(c: &char) -> Option<WordType> {
//         match c {
//             c if c.is_whitespace() => Some(WordType::WhiteSpace),
//             c if c.is_ascii_punctuation() => Some(WordType::Punctuation),
//             c if c.is_alphanumeric() => Some(WordType::AlphaNumeric),
//             _ => None,
//         }
//     }
// }

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
            line.insert(cursor.0 as usize, c);
        }
    }

    pub fn remove(&mut self, y: usize) {
        if self.lines.get_mut(y).is_some() {
            self.lines.remove(y);
        }
    }

    pub fn remove_char(&mut self, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
            line.remove(cursor.0 as usize);
        }
    }

    pub fn remove_word(&mut self, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
            let chars: Vec<char> = line.clone().chars().collect();
            let mut i = cursor.0 as usize;

            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }

            let mut word_end = i;
            while word_end < chars.len() && chars[word_end].is_alphanumeric()
                || chars[word_end] == '_'
            {
                word_end += 1;
            }

            line.replace_range(i..word_end, "");
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

// TEST de remove_word
//            // if line.is_empty() {
//     return;
// }
// // je recois une
// let x = cursor.0 as usize;
//
// // i think this cant not fail because we leave if empty and cursor cant be on non char
// // in term
// let slice = &line.clone()[x..];
// let first_char = &line.clone()[x..x + 1].chars().next().unwrap();
//
// if let Some(c_type) = WordType::get_type(&first_char) {
//     log_message!("type: {:?}", c_type);
//     let first_non_matched = match c_type {
//         WordType::WhiteSpace => {
//             slice.find(|c: char| c.is_alphanumeric() || c.is_ascii_punctuation())
//         }
//         WordType::Punctuation => {
//             slice.find(|c: char| c.is_whitespace() || c.is_alphanumeric())
//         }
//         WordType::AlphaNumeric => {
//             slice.find(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
//         }
//     };
//
//     let end_of_line = match first_non_matched {
//         Some(index) => &slice[index..],
//         None => &slice,
//     };
//
//     let mut res = String::from(&line[..x]);
//
//     res.push_str(end_of_line);
//     line.clear();
//     line.push_str(res.as_str());
// }
// // log_message!("{:?}, line: {}", c_type, line);
// // let l_type = line.clone()[x..].chars();
