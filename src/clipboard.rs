use std::io;

pub const CLIPBOARD_WARN_BYTES: u64 = 500 * 1024;

pub fn copy_to_clipboard(content: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let cmd = "pbcopy";
    #[cfg(target_os = "linux")]
    let cmd = "xclip -selection clipboard";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Clipboard not supported on this platform.".to_string());

    use std::process::{Command, Stdio};
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch clipboard command: {}", e))?;
    if let Some(mut stdin) = child.stdin.take() {
        io::Write::write_all(&mut stdin, content.as_bytes())
            .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
    }
    child.wait().map_err(|e| format!("Clipboard command failed: {}", e))?;
    Ok(())
}
