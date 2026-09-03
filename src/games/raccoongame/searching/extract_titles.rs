use crate::games::raccoongame::searching::meilisearch::GAME;
use crate::games::raccoongame::searching::pull_games::yoit;
use crate::games::raccoongame::raccoon_account::full_account_builder::build;
use crate::logs::write::raccoon::database::accountdb::db::account_db;

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

    tokio::spawn(classify_catalog());

    Ok(parsed_games)
}

async fn classify_catalog() {
    let db = match account_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("couldnt open accountdb for the probe: {}", e);
            return;
        }
    };
    if db.games_count().unwrap_or(0) > 0 {
        println!("game catalog already probed, skipping");
        return;
    }
    println!("probing game catalog, this makes a fresh account and hits raccoon");
    match build().await {
        Ok((token, sn, _, _)) => match crate::games::raccoongame::play::probe::classify(&token, &sn).await {
            Ok(playable) => println!("probe done, {} playable games", playable),
            Err(e) => eprintln!("probe failed: {}", e),
        },
        Err(e) => eprintln!("couldnt build a throwaway account: {}", e),
    }
}