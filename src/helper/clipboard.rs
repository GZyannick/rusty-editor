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
        let mut lines: Vec<String> = content.split('\n').map(|v| v.to_string()).collect();
        // pop the last \n to not have an empty line
        if let Some(last) = lines.last() {
            if last.is_empty() && lines.len() > 1 {
                lines.pop();
            }
        }
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
    let output = Command::new("pbpaste").output().ok()?;
    let clipboard = String::from_utf8(output.stdout).ok();
    if let Some(content) = clipboard {
        let mut lines: Vec<String> = content.split('\n').map(|v| v.to_string()).collect();

        //pop the last \n to not have an empty line
        if let Some(last) = lines.last() {
            if last.is_empty() && lines.len() > 1 {
                lines.pop();
            }
        }
        return Some(lines);
    }
    None
}

#[cfg(target_os = "windows")]
fn copy_to_clipboard(data: &str) {
    extern crate winapi;

    use std::ffi::CString;
    use std::ptr::null_mut;
    use winapi::shared::minwindef::*;
    use winapi::um::winbase::*;
    use winapi::um::winuser::*;

    unsafe {
        // Open the clipboard
        if OpenClipboard(null_mut()) == 0 {
            eprintln!("Failed to open clipboard");
            return;
        }

        // Empty the clipboard
        EmptyClipboard();

        // Allocate global memory
        let data = CString::new(data).unwrap();
        let len = data.as_bytes_with_nul().len();
        let h_mem = GlobalAlloc(GMEM_MOVEABLE, len);

        if h_mem.is_null() {
            eprintln!("Failed to allocate memory");
            CloseClipboard();
            return;
        }

        // Copy data into global memory
        let ptr = GlobalLock(h_mem) as *mut u8;
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, len);
        GlobalUnlock(h_mem);

        // Set the clipboard data
        SetClipboardData(CF_TEXT, h_mem);

        // Close the clipboard
        CloseClipboard();
    }
}

#[cfg(target_os = "windows")]
fn paste_from_clipboard() -> Option<Vec<String>> {
    unsafe {
        // Open the clipboard
        if OpenClipboard(null_mut()) == 0 {
            eprintln!("Failed to open clipboard");
            return None;
        }

        // Get clipboard data
        let h_mem = GetClipboardData(CF_TEXT);
        if h_mem.is_null() {
            eprintln!("Failed to get clipboard data");
            CloseClipboard();
            return None;
        }

        // Lock and read the memory
        let ptr = GlobalLock(h_mem) as *const u8;
        let mut len = 0;

        while *ptr.add(len) != 0 {
            len += 1;
        }

        let clipboard = String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string();
        GlobalUnlock(h_mem);

        // Close the clipboard
        CloseClipboard();
        if let Some(content) = clipboard {
            let mut lines: Vec<String> = content.split('\n').map(|v| v.to_string()).collect();
            // pop the last \n to not have an empty line
            if let Some(last) = lines.last() {
                if let Some(last) = lines.last() {
                    if last.is_empty() && lines.len() > 1 {
                        lines.pop();
                    }
                }
            }
            return Some(lines);
        }
        None
    }
}
