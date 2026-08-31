use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::time::Duration;
use crate::games::raccoongame::limiter::RACCOON_GAME_LIMITER;

//send captcja
pub async fn send_email(email: String, sn: String) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let mut last_status = String::from("no response");
    for attempt in 0..3 {
        crate::dlog!("[send_email] inputs: url=https://www.raccoongame.com/users/sendEmail attempt={}/3 email={} type=register sn={}", attempt + 1, email, sn);
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"));
        headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"));
        headers.insert("origin", HeaderValue::from_static("https://www.raccoongame.com"));
        headers.insert("priority", HeaderValue::from_static("u=1, i"));
        headers.insert("referer", HeaderValue::from_static("https://www.raccoongame.com/login?redirect_uri=https%3A%2F%2Fwww.raccoongame.com%2Fweb2%2Fdist%2F%23%2Fplatform%2Fcloudgame"));
        headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"150\", \"Google Chrome\";v=\"150\""));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Linux\""));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36"));
        headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));

        let form_params = [
            ("email", email.as_str()),
            ("type", "register"),
            ("sn", sn.as_str()),
            ("model", "Chrome/150.0.0.0"),
            ("version_code", "1"),
            ("version_name", "1.0.0"),
            ("device_name", "我的电脑"),
            ("os", "pc"),
        ];

        RACCOON_GAME_LIMITER.until_ready().await;
        let response = client.post("https://www.raccoongame.com/users/sendEmail")
            .headers(headers)
            .form(&form_params)
            .send()
            .await;
        match &response {
            Ok(res) => crate::dlog!("[send_email] output: http status={}", res.status()),
            Err(e) => crate::dlog!("[send_email] output: reqwest errored: {}", e),
        }

        if let Ok(res) = response {
            last_status = res.status().to_string();
            match res.json::<Value>().await {
                Ok(response_json) => {
                    crate::dlog!("[send_email] output body: {}", response_json);
                    if response_json["status"].as_i64() == Some(200) {
                        return Ok(());
                    }
                }
                Err(e) => crate::dlog!("[send_email] json parse falled: {}", e),
            }
        }

        if crate::games::raccoongame::limiter::RETRY_BACKOFF_ENABLED {

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        }
    }
    anyhow::bail!("failed to send email (last http status: {})", last_status);
}