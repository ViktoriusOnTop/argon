use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;
use crate::games::raccoongame::limiter::RACCOON_GAME_LIMITER;
use crate::games::raccoongame::raccoon_account::full_account_builder::build;

pub async fn yoit() -> anyhow::Result<Vec<Value>> {
    for _ in 0..3 {
        if let Ok((token, sn, _, _)) = build().await {
            if let Ok(games) = fetch_pages(&token, &sn).await {
                return Ok(games);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("pullin games failed")
}

async fn fetch_pages(token: &str, sn: &str) -> anyhow::Result<Vec<Value>> {
    let mut all: Vec<Value> = Vec::new();
    let mut seen: HashMap<u64, ()> = HashMap::new();
    let mut page: u32 = 1;

    loop {
        let value = fetch_page(token, sn, page).await?;
        let Some(list) = value["data"]["data"].as_array() else {
            break;
        };
        if list.is_empty() {
            break;
        }

        let total = value["data"]["total"].as_u64().unwrap_or(0);
        for item in list {
            if let Some(id) = item["id"].as_u64() {
                if seen.insert(id, ()).is_none() {
                    all.push(item.clone());
                }
            }
        }

        if (all.len() as u64) >= total {
            break;
        }
        page += 1;
        if page > 500 {
            break;
        }
    }
    Ok(all)
}

async fn fetch_page(token: &str, sn: &str, page: u32) -> anyhow::Result<Value> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut params = HashMap::new();
    params.insert("sn", sn.to_string());
    params.insert("model", "Chrome/150.0.0.0".to_string());
    params.insert("version_code", "1".to_string());
    params.insert("version_name", "1.0.0".to_string());
    params.insert("device_name", "我的设备".to_string());
    params.insert("os", "web".to_string());
    params.insert("manufacturer;", String::new());
    params.insert("page", page.to_string());
    params.insert("page_size", "20".to_string());
    params.insert("game_name", String::new());
    params.insert("platform", "3".to_string());
    params.insert("user_token", token.to_string());

    let cookie = format!("as_user_token={}", token);

    crate::vlog!("[yoit] inputs: url=https://www.raccoongame.com/game/gameList page={} cookie={} params={:?}", page, cookie, params);

    RACCOON_GAME_LIMITER.until_ready().await;
    let response = client
        .post("https://www.raccoongame.com/game/gameList")
        .header("accept", "*/*")
        .header("accept-language", "en-US,en;q=0.9")
        .header("origin", "https://www.raccoongame.com")
        .header("priority", "u=1, i")
        .header("referer", "https://www.raccoongame.com/?t=1720436119")
        .header("sec-ch-ua", r#""Not;A=Brand";v="8", "Chromium";v="150", "Google Chrome";v="150""#)
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", r#""Linux""#)
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36")
        .header("x-requested-with", "XMLHttpRequest")
        .header("cookie", cookie)
        .form(&params)
        .send()
        .await;

    match &response {
        Ok(res) => crate::vlog!("[yoit] output: http status={}", res.status()),
        Err(e) => crate::vlog!("[yoit] output: reqwest errored: {}", e),
    }
    if let Ok(response) = response {
        match response.json::<Value>().await {
            Ok(response_json) => {
                crate::vlog!("[yoit] output body: {}", response_json);
                return Ok(response_json);
            }
            Err(e) => crate::vlog!("[yoit] json parse falled: {}", e),
        }
    }
    anyhow::bail!("yoitin game page {} failed", page)
}
