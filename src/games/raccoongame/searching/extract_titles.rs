use crate::games::raccoongame::searching::meilisearch::GAME;
use crate::games::raccoongame::searching::pull_games::yoit;

pub async fn extract() -> anyhow::Result<Vec<GAME>> {
    for _ in 0..3 {
        if let Ok(games) = yoit().await {
            if let Some(list) = games["data"]["data"].as_array() {
                let parsed_games = list
                    .iter()
                    .filter_map(|item| {
                        Some(GAME {
                            id: item["id"].as_u64()?,
                            name: item["game_name"].as_str()?.to_string(),
                            main_img: item["game_main_img"].as_str()?.to_string(),
                            game_recommend_img: item["game_recommend_img"].as_str()?.to_string(),
                            game_icon: item["game_icon"].as_str()?.to_string(),
                        })
                    })
                    .collect();
                return Ok(parsed_games);
            }
        }
    }
    anyhow::bail!("Yoit failed thrice, extractions hard to fail so prob not that")
}