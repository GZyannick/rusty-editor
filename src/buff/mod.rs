use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    ops::{RangeFrom, RangeTo},
    path::PathBuf,
    str::FromStr,
};

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

    pub fn get(&self, n: usize) -> Option<String> {
        self.lines.get(n).cloned()
    }

    pub fn get_block(&self, start: (u16, u16), end: (u16, u16)) -> Vec<Option<String>> {
        let mut block: Vec<Option<String>> = vec![];
        let mut i = start.1;

        while i <= end.1 {
            let mut opt_line = self.get(i as usize).clone();
            if let Some(line) = &opt_line {
                match i {
                    x if x == start.1 => {
                        opt_line = Some(line[start.0 as usize..].to_string());
                    }
                    x if x == end.1 => {
                        opt_line = Some(line[..end.0 as usize + 1].to_string());
                    }
                    _ => {}
                };
            }
            block.push(opt_line);
            i += 1;
        }
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

    fn drain_and_copy_line_to(
        &mut self,
        line: &str,
        index: usize,
        range: RangeTo<usize>,
    ) -> Option<String> {
        let line = Some(line[range].to_string());
        self.lines.get_mut(index).unwrap().drain(range);
        line
    }

    // i dont like to have two times the nearly the same fn but
    // line[range] doenst take RangeBounds
    // and add Into<Option<RangeFrom<usize> + Into<Option<RangeTo<usize>>>>> make it over
    // complicated for nothing
    fn drain_and_copy_line_from(
        &mut self,
        line: &str,
        index: usize,
        range: RangeFrom<usize>,
    ) -> Option<String> {
        // PS this is funny that RangeTo as the trait Copy but not RangeFrom<>
        let line = Some(line[range.clone()].to_string());
        self.lines.get_mut(index).unwrap().drain(range);
        line
    }

    pub fn remove_block(&mut self, start: (u16, u16), end: (u16, u16)) -> Vec<Option<String>> {
        let mut block: Vec<Option<String>> = vec![];
        let mut to_remove_index: Vec<usize> = vec![];

        let mut i = start.1;
        while i <= end.1 {
            let mut opt_line = self.get(i as usize).clone();

            // check if we remove the line or drain it
            if let Some(line) = &opt_line {
                match i {
                    // drain at start and end line
                    x if x == start.1 => {
                        opt_line =
                            self.drain_and_copy_line_from(line, x as usize, start.0 as usize..);
                    }
                    x if x == end.1 => {
                        opt_line =
                            self.drain_and_copy_line_to(line, x as usize, ..end.0 as usize + 1);
                    }
                    x => {
                        to_remove_index.push(x as usize);
                    }
                };
            }
            block.push(opt_line);
            i += 1;
        }

        // we remove block after because we dont want to iterate on lines and remove at the same
        // time
        for index in to_remove_index {
            self.remove(index);
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
}
