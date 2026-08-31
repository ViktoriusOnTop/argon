use std::collections::VecDeque;
use std::fs;

use crate::games::raccoongame::pool::account::ACCOUNT;
use crate::logs::write::raccoon::pool::save_pool::pool_file_path;

pub fn load_pool() -> VecDeque<ACCOUNT> {
    let Ok(path) = pool_file_path() else {
        return VecDeque::new();
    };
    let Ok(data) = fs::read_to_string(path) else {
        return VecDeque::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}
