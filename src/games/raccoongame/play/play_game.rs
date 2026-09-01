use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

use crate::games::raccoongame::limiter::RACCOON_GAME_LIMITER;

pub async fn play_game(token: &str, sn: &str, game_key: &str, queue_id: Option<&str>) -> anyhow::Result<Value> {
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
    params.insert("game_key", game_key.to_string());
    params.insert("model_name", "Chrome/150.0.0.0".to_string());
    params.insert("user_token", token.to_string());
    if let Some(qid) = queue_id {
        params.insert("play_queue_id", qid.to_string());
    }

    let cookie = format!("as_user_token={}", token);

    crate::vlog!("[play_game] inputs: url=https://www.raccoongame.com/jyapi/playGame cookie={} params={:?}", cookie, params);

    RACCOON_GAME_LIMITER.until_ready().await;
    let response = client
        .post("https://www.raccoongame.com/jyapi/playGame")
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
        Ok(res) => crate::vlog!("[play_game] output: http status={}", res.status()),
        Err(e) => crate::vlog!("[play_game] output: reqwest errored: {}", e),
    }
    if let Ok(response) = response {
        match response.json::<Value>().await {
            Ok(response_json) => {
                crate::vlog!("[play_game] output body: {}", response_json);
                return Ok(response_json);
            }
            Err(e) => crate::vlog!("[play_game] json parse falled: {}", e),
        }
    }
    anyhow::bail!("playGame failed for game_key {} is thoust stupid and gave wrong key?", game_key)
}