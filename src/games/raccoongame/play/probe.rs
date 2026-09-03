use serde::{Deserialize, Serialize};

use crate::games::raccoongame::play::check_cost::check_cost;
use crate::logs::write::raccoon::database::accountdb::db::account_db;

pub const PLAYABLE_STATUS: u64 = 200;
pub const MEMBERSHIP_STATUS: u64 = 4623;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameProbe {
    pub name: String,
    pub game_key: String,
    pub playable: bool,
}

pub fn seed() -> Vec<GameProbe> {
    let path = crate::logs::make_folder::get_platform_log_dir()
        .map(|d| d.join("games").join("game_keys.json"))
        .unwrap_or_else(|_| std::path::PathBuf::from("game_keys.json"));
    crate::vlog!("[probe] seed inputs: path={}", path.display());
    let s = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            crate::vlog!("[probe] seed output: read failed path={} err={}", path.display(), e);
            return Vec::new();
        }
    };
    let v = serde_json::from_str::<Vec<GameProbe>>(&s).unwrap_or_default();
    crate::vlog!("[probe] seed output: parsed {} games", v.len());
    v
}

pub async fn classify(token: &str, sn: &str) -> anyhow::Result<usize> {
    let db = account_db()?;
    let mut results = Vec::new();

    for game in seed() {
        let playable = match check_cost(token, sn, &game.game_key).await {
            Ok(v) => {
                let status = v["status"].as_u64().unwrap_or(0);
                crate::vlog!(
                    "[probe] {} status={} playable={}",
                    game.game_key,
                    status,
                    status == PLAYABLE_STATUS
                );
                status == PLAYABLE_STATUS
            }
            Err(e) => {
                eprintln!("[probe] {} check failed: {}", game.game_key, e);
                false
            }
        };
        results.push(GameProbe {
            name: game.name,
            game_key: game.game_key,
            playable,
        });
    }

    db.set_games(&results)?;
    let playable = results.iter().filter(|g| g.playable).count();
    crate::vlog!("[probe] classified {} games, {} playable", results.len(), playable);
    Ok(playable)
}