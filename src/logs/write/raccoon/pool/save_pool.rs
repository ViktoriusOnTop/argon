use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use crate::games::raccoongame::pool::account::ACCOUNT;
use crate::logs::make_folder::get_platform_log_dir;

pub fn pool_file_path() -> anyhow::Result<PathBuf> {
    Ok(get_platform_log_dir()?
        .join("games")
        .join("raccoon")
        .join("pool.json"))
}

pub fn save_pool(accounts: &VecDeque<ACCOUNT>) -> anyhow::Result<()> {
    let path = pool_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string(accounts)?)?;
    Ok(())
}
