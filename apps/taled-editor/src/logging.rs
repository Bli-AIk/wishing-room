use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::sync::OnceLock;

static LOG_PATH: OnceLock<String> = OnceLock::new();

/// Initialize file logging.  Truncates any previous log.
pub(crate) fn init(dir: &str) {
    let log_dir = format!("{dir}/logs");
    let _ = create_dir_all(&log_dir);
    let path = format!("{log_dir}/taled.log");
    if let Ok(mut f) = std::fs::File::create(&path) {
        let _ = writeln!(f, "[taled] log session started");
    }
    let _ = LOG_PATH.set(path);
}

/// Append a single line to the log file.
pub(crate) fn append(msg: &str) {
    let Some(path) = LOG_PATH.get() else { return };
    let Ok(mut f) = OpenOptions::new().append(true).create(true).open(path) else {
        return;
    };
    let _ = writeln!(f, "{msg}");
}

/// Log directory path (for display to the user).
pub(crate) fn log_path() -> Option<&'static str> {
    LOG_PATH.get().map(String::as_str)
}
