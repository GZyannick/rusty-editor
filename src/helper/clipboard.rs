use std::process::Command;
#[cfg(target_os = "linux")]
pub fn copy_to_clipboard(data: &str) {
    let _ = Command::new("xclip")
        .args(&["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(data.as_bytes())
        });
}

#[cfg(target_os = "linux")]
pub fn paste_from_clipboard() -> Option<Vec<String>> {
    let output = Command::new("xclip")
        .args(&["-selection", "clipboard", "-o"])
        .output()
        .ok()?;
    let clipboard = String::from_utf8(output.stdout).ok();
    if let Some(content) = clipboard {
        let lines: Vec<String> = content.split('\n').map(|v| v.to_string()).collect();
        return Some(lines);
    }
    None
}

#[cfg(target_os = "macos")]
pub fn copy_to_clipboard(data: &str) {
    let _ = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(data.as_bytes())
        });
}

#[cfg(target_os = "macos")]
pub fn paste_from_clipboard() -> Option<Vec<String>> {
    use crate::log_message;

    let output = Command::new("pbpaste").output().ok()?;
    let clipboard = String::from_utf8(output.stdout).ok();
    if let Some(content) = clipboard {
        log_message!("clipboard: {:?}", content);
        let mut lines: Vec<String> = content.split('\n').map(|v| v.to_string()).collect();
        // pop the last \n to not have an empty line
        if let Some(last) = lines.last() {
            if last.is_empty() {
                lines.pop();
            }
        }
        return Some(lines);
    }
    None
}
