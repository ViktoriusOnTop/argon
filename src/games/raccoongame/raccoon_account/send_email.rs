use rand::distr::Alphanumeric;
use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Response;
use serde_json::Value;

async fn send_email(email: String, sn: String) -> anyhow::Result<()> {
    for _ in 0..3{
        let client = reqwest::Client::new();
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

        let response = client.post("https://www.raccoongame.com/users/sendEmail")
            .headers(headers)
            .form(&form_params)
            .send()
            .await;

        if let Ok(res) = response {
            if let Ok(response_json) = res.json::<Value>().await {
                if response_json["status"].as_i64() == Some(200) {
                    return Ok(());
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Failed to send email");
}