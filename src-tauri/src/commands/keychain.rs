use keyring::Entry;

#[tauri::command]
pub fn store_secret(service: String, key: String, value: String) -> Result<(), String> {
    Entry::new(&service, &key)
        .map_err(|e| e.to_string())?
        .set_password(&value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_secret(service: String, key: String) -> Result<Option<String>, String> {
    match Entry::new(&service, &key).map_err(|e| e.to_string())?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn delete_secret(service: String, key: String) -> Result<(), String> {
    match Entry::new(&service, &key).map_err(|e| e.to_string())?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
