pub const CLIPBOARD_WARN_BYTES: u64 = 500 * 1024;

pub fn copy_to_clipboard(content: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .map_err(|e| format!("Failed to open clipboard: {}", e))?
        .set_text(content)
        .map_err(|e| format!("Failed to write to clipboard: {}", e))
}
