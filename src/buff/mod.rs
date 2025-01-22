use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    ops::Range,
    path::PathBuf,
    str::FromStr,
};

use crate::log_message;

#[derive(Debug)]
pub struct Buffer {
    pub file: Option<File>,
    pub is_directory: bool,
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
            is_directory: false,
            lines: vec![String::new()],
            path: "Empty".to_string(),
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

        Buffer {
            file,
            is_directory: false,
            lines,
            path,
        }
    }

    fn from_dir(path: &str) -> Buffer {
        let d_path = path.to_string();
        let mut lines: Vec<String> = vec![String::from("../")];
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                let path = entry.unwrap().path();
                if let Some(path_str) = path.to_str() {
                    lines.push(String::from(path_str));
                }
            }
        }

        Buffer {
            file: None,
            is_directory: true,
            lines,
            path: d_path,
        }
    }

    pub fn parent_dir(&mut self) -> Option<Buffer> {
        match PathBuf::from_str(&self.path) {
            Ok(path_buf) => match path_buf.parent() {
                Some(parent_path) => {
                    let parent_path = parent_path.to_str().unwrap().to_string();
                    // sometimes path_buf.parent return an empty so we check because
                    // we cannot have an empty path in a viewport
                    if parent_path.is_empty() {
                        return None;
                    }
                    Some(Buffer::new(Some(parent_path)))
                }
                None => None,
            },

            Err(_) => {
                // we didnt make the error follow because the error
                // is to say we are at the original path of file_directory
                None
            }
        }
    }

    pub fn _get_char(&self, cursor: &(u16, u16)) -> Option<char> {
        if let Some(line) = self.get(cursor.1 as usize) {
            return line.chars().nth(cursor.0 as usize);
        }
        None
    }
    pub fn get(&self, n: usize) -> Option<String> {
        self.lines.get(n).cloned()
    }

    pub fn get_block(&self, start: (u16, u16), end: (u16, u16)) -> Option<String> {
        let mut block: Option<String> = None;
        let mut i = start.1;

        while i <= end.1 {
            if let Some(line) = self.get(i as usize).clone() {
                let mut modified_line = match i {
                    // if its the first line
                    x if x == start.1 => line[start.0 as usize..].to_string(),
                    x if x == end.1 => {
                        let end_x = match line.is_empty() {
                            true => end.0 as usize,
                            false => end.0 as usize + 1,
                        };
                        line[..end_x].to_string()
                    }
                    _ => line.to_string(),
                };

                if !line.is_empty() {
                    modified_line.push('\n');
                }
                match &mut block {
                    Some(content) => content.push_str(&modified_line),
                    None => block = Some(String::from(&modified_line)),
                }
            }

            i += 1;
        }

        // while i <= end.1 {
        //     let mut opt_line = self.get(i as usize).clone();
        //     if let Some(line) = &opt_line {
        //         match i {
        //             x if x == start.1 => {
        //                 opt_line = Some(line[start.0 as usize..].to_string());
        //             }
        //             x if x == end.1 => {
        //                 // we add 1 because the range will not take the last char
        //                 let end_x = match line.is_empty() {
        //                     true => end.0 as usize,
        //                     false => end.0 as usize + 1,
        //                 };
        //                 opt_line = Some(line[..end_x].to_string());
        //             }
        //             _ => {}
        //         };
        //     }
        //     block.push(opt_line);
        //     i += 1;
        // }
        //
        block
    }

    pub fn new_line(&mut self, cursor: (u16, u16), is_take_text: bool) {
        let y_pos: usize = cursor.1 as usize;
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

    fn drain_and_copy(
        &mut self,
        line: &str,
        index: usize,
        range: Range<usize>,
        is_last_line: bool,
    ) -> (Option<String>, bool) {
        // copy line
        let mut line = line[range.clone()].to_string();
        // get mutable line vec of lines
        let mut_line = self.lines.get_mut(index).unwrap();
        mut_line.drain(range);

        // we check if the last line is empty and add a \n to know when we undo if the last lane
        // need to be insert in a existed string
        if is_last_line && mut_line.is_empty() {
            line.push('\n');
        }
        (Some(line), mut_line.is_empty())
    }

    pub fn remove_block(
        &mut self,
        start: (u16, u16),
        end: (u16, u16),
        remove_first_line: bool,
    ) -> Vec<Option<String>> {
        let mut block: Vec<Option<String>> = vec![];
        let mut to_remove_index: Vec<usize> = vec![];
        let mut is_last_line = false;

        let mut i = start.1;
        while i <= end.1 {
            let mut opt_line = self.get(i as usize).clone();
            // check if we remove the line or drain it
            if let Some(line) = &opt_line {
                match i > start.1 && i < end.1 {
                    true => to_remove_index.push(i as usize), // remove it if its not the first or
                    // last line
                    false => {
                        let end_x = match line.is_empty() {
                            true => end.0 as usize,
                            false => end.0 as usize + 1,
                        };
                        let range: Range<usize> = match i {
                            x if x == start.1 && x == end.1 => start.0 as usize..end_x,
                            x if x == start.1 => start.0 as usize..line.len(),
                            _ => {
                                is_last_line = true;
                                0..end_x
                            } // x is forcely equal to end.1 we tried
                              // all other possibility
                        };
                        let (cp_line, is_empty) =
                            self.drain_and_copy(line, i as usize, range, is_last_line);
                        opt_line = cp_line;

                        match remove_first_line {
                            true if is_last_line && is_empty => to_remove_index.push(i as usize),
                            false => to_remove_index.push(i as usize),
                            _ => (),
                        }

                        // if is_empty {
                        //     to_remove_index.push(i as usize);
                        // }
                    }
                }
            }
            block.push(opt_line);
            i += 1;
        }

        // we remove block after because we dont want to iterate on lines and remove at the same
        // time
        while !to_remove_index.is_empty() {
            if let Some(index) = to_remove_index.pop() {
                self.remove(index);
            }
        }

        block
    }

    pub fn remove_char(&mut self, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize) {
            line.remove(cursor.0 as usize);
        }
    }

    pub fn remove_word(&mut self, _cursor: (u16, u16)) {
        todo!()
    }

    pub fn remove_char_line(&mut self, cursor: (u16, u16)) {
        let mut buf = String::new();
        if let Some(line) = self.get(cursor.1 as usize) {
            buf = line.clone();
            self.lines.remove(cursor.1 as usize);
        }
        if let Some(prev_line) = self.lines.get_mut(cursor.1 as usize - 1) {
            prev_line.push_str(buf.as_str());
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

    // return a bool to know if the file is save
    // only compare file and not the file_explorer
    pub fn compare_file(&mut self) -> anyhow::Result<bool> {
        if self.is_directory {
            return Ok(false);
        }

        if let Ok(mut c_file) = File::open(&self.path) {
            let mut buf = String::new();
            c_file.read_to_string(&mut buf).unwrap();

            let lines = buf.lines().map(|s| s.to_string()).collect::<Vec<String>>();

            // let matching = lines.iter().zip(&self.lines).filter(|&(a, b)| a == b);
            for (a, b) in lines.iter().zip(&self.lines) {
                if a != b {
                    // not saved
                    return Ok(true);
                }
            }
        }
        // if let Some(c_file) = &mut self.file {}
        // no diff between file
        Ok(false)
    }

    pub fn push_or_insert(&mut self, line: String, y: usize) {
        match y >= self.lines.len() {
            true => self.lines.push(line),
            false => self.lines.insert(y, line),
        }
    }

    pub fn insert_str(&mut self, y: usize, x: usize, content: &str) {
        if let Some(buffer_line) = self.lines.get_mut(y) {
            buffer_line.insert_str(x, content);
        }
    }
}
