use std::collections::HashMap;
use serde_json::Value;
use crate::games::raccoongame::limiter::RACCOON_GAME_LIMITER;
use crate::games::raccoongame::raccoon_account::full_account_builder::build;

async fn yoit() -> anyhow::Result<Value>{
    for _ in 0..3 {
        if let Ok(boosh) = build().await{
            let (token, sn) = boosh;

            let client = reqwest::Client::new();

            let url = "https://www.raccoongame.com/game/gameList";

            let mut params = HashMap::new();
            params.insert("sn", sn.as_str());
            params.insert("model", "Chrome/150.0.0.0");
            params.insert("version_code", "1");
            params.insert("version_name", "1.0.0");
            params.insert("device_name", "我的设备");
            params.insert("os", "web");
            params.insert("manufacturer;", "");
            params.insert("page", "1");
            params.insert("page_size", "20");
            params.insert("game_name", "");
            params.insert("platform", "3");
            params.insert("user_token", token.as_str());

            let cookie = format!("as_user_token={}", token.as_str());

            RACCOON_GAME_LIMITER.until_ready().await;
            if let Ok(response) = client
                .post(url)
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
                .header(
                    "cookie",
                    cookie
                )
                .form(&params)
                .send()
                .await{
                if let Ok(response_json) = response.json::<Value>().await {
                    return Ok(response_json);
                }
            }
        }
    }
    anyhow::bail!("Yoiting games failed")
}