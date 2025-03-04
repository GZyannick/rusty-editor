use std::path::PathBuf;

pub fn get_icon(path: &String) -> &'static str {
    match PathBuf::from(path).is_dir() {
        true => " \u{f115}",
        false => match path.split('.').last() {
            Some("txt") => " \u{f15c}",
            Some("md") => " \u{f48a}",
            Some("rs") => " \u{e7a8}",
            Some("py") => " \u{e73c}",
            Some("png") | Some("jpg") => " \u{f1c5}",
            _ => " \u{f016}",
        },
    }
}
