use std::{fs::{File, OpenOptions}, io::Write, io::Read, usize};

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

        Buffer { file: None, lines: vec![], path: "Empty".to_string() }
    }

    pub fn get_line(&self, n: usize) -> Option<String> {
        self.lines.get(n).cloned()
    }

    pub fn add_char(&mut self, c: char, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize ) {
            line.insert(cursor.0 as usize - 1_usize, c);
        }
    }

    pub fn remove_char(&mut self, cursor: (u16, u16)) {
        if let Some(line) = self.lines.get_mut(cursor.1 as usize ) {
            line.remove(cursor.0 as usize - 1_usize);
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
            lines,
            path,
        }
    }

    fn from_dir(_f_path: &str) -> Buffer {
        //TODO Handle dir path for now it will return an empty buffer
        Buffer { file: None, lines: vec![], path: "Empty".to_string() }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {

        // TODO handle file creation if he doenst exist;
        if let Some(_c_file) = &self.file {
            let mut open_file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(self.path.clone())?;
            for line in self.lines.iter() {
                writeln!(open_file, "{line}")?;
            }

        }
        
        Ok(())
    }
}
