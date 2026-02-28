#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error in {context}: {source}")]
    Db {
        context: &'static str,
        #[source]
        source: rusqlite::Error,
    },
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("not configured: {0}")]
    NotConfigured(&'static str),
}

// Allow Tauri to convert AppError to String for IPC responses
impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}
