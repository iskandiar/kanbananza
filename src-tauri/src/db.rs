/// Wraps a rusqlite `Connection` in a `Mutex` so Tauri can share it across
/// command handlers as managed state. All commands lock this mutex for the
/// duration of their query — there is no connection pool.
use rusqlite::Connection;
use std::sync::Mutex;

pub struct DbState(pub Mutex<Connection>);
