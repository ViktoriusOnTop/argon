use crate::games::raccoongame::searching::meilisearch::GAME;
use crate::games::raccoongame::searching::pull_games::yoit;

pub async fn extract() -> anyhow::Result<Vec<GAME>> {
    let games = yoit().await?;
    let parsed_games = games
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
    Ok(parsed_games)
}