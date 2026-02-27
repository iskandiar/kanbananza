#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(format!("rejected non-http URL: {url}"));
    }
    open::that(&url).map_err(|e| format!("failed to open URL: {e}"))
}
