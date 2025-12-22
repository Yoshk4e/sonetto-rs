use once_cell::sync::Lazy;
use std::path::PathBuf;

pub mod time;

pub const HOST: &str = "127.0.0.1";
pub const DNS: &str = "localhost";
pub const HTTPSERVER_PORT: u16 = 21000;
pub const GAMESERVER_PORT: u16 = 23301;

pub const CERT_DIR: &str = "./cert";
pub const KEY_FILE_PATH: &str = "./cert/localhost.key";
pub const CERT_FILE_PATH: &str = "./cert/localhost.crt";

// todo: user regist n allat shit
pub const USER_ID: u64 = 1337;
pub const USERNAME: &str = "Yon";

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().unwrap();

    tracing_subscriber::fmt().init();
}

use std::time::{SystemTime, UNIX_EPOCH};

pub fn cur_time_ms_u128() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub static DATA_DIRECTORY: Lazy<PathBuf> = Lazy::new(|| {
    // 1. Check environment variable first
    if let Ok(data_dir) = std::env::var("DATA_DIR") {
        let path = PathBuf::from(data_dir);
        if path.exists() {
            return path;
        }
    }

    // 2. Check current working directory
    let current_dir_data = std::env::current_dir()
        .ok()
        .map(|p| p.join("..\\..\\data\\static"))
        .filter(|p| p.exists());
    if let Some(path) = current_dir_data {
        return path;
    }

    // 3. Check executable's directory
    let exe_dir_data = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("..\\..\\data\\static")))
        .filter(|p| p.exists());
    if let Some(path) = exe_dir_data {
        return path;
    }

    // 4. Fallback to cargo manifest dir for development
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_path = PathBuf::from(manifest_dir).join("..\\..\\data\\static");
        if dev_path.exists() {
            return dev_path;
        }
    }

    // 5. Last resort - return current_dir/data even if it doesn't exist
    // (will give clear error messages when files are accessed)
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("..\\..\\data\\static")
});

pub fn time_ms_u64() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
