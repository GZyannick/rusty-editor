#[derive(Debug, Copy, Clone)]
pub enum CharType {
    Alphabetic,
    Numeric,
    Whitespace,
    AsciiPunctuation,
    None,
}
impl CharType {
    pub fn new(c: &char) -> Self {
        match c {
            ch if ch.is_alphabetic() => Self::Alphabetic,
            ch if ch.is_numeric() => Self::Numeric,
            ch if ch.is_whitespace() => Self::Whitespace,
            ch if ch.is_ascii_punctuation() => Self::AsciiPunctuation,
            _ => Self::None,
        }
    }

    // if Some(base_len) mean that we go to next different type
    // and if None its prev
    pub fn goto_diff_type(line: String, base_len: Option<u16>, cursor_x: &mut u16) {
        let mut base_type = CharType::None;
        for (i, c) in line.chars().enumerate() {
            let char_type = CharType::new(&c);
            if i == 0 {
                base_type = char_type;
                continue;
            }
            if !base_type.eq(&char_type) {
                // because its a range we have to take in account the size of the line
                // not the size of the range that why we do += x
                let x = match char_type {
                    CharType::Whitespace => match line.chars().nth(i + 1) {
                        Some(_) => i as u16 + 1, // we want the char behind the first
                        // whitespace
                        None => i as u16,
                    },
                    _ => i as u16,
                };

                match base_len.is_some() {
                    true => *cursor_x += x,
                    false => *cursor_x = cursor_x.saturating_sub(x),
                }
                break;
            } else if base_type.eq(&char_type) && i == line.len() - 1 {
                *cursor_x = base_len.unwrap_or(0)
            }
        }
    }
}

impl std::cmp::PartialEq for CharType {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}
