use crate::games::raccoongame::searching::pull_games::yoit;

pub async fn extract() -> anyhow::Result<Vec<String>> {
    for _ in 0..3 {
        if let Ok(games) = yoit().await {
            if let Some(list) = games["data"]["data"].as_array() {
                let game_names = list
                    .iter()
                    .filter_map(|item| item["game_name"].as_str().map(String::from))
                    .collect();
                return Ok(game_names);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Yoit failed thrice, extractions hard to fail so prob not that")
}