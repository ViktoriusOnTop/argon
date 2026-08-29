use rand::distr::Alphanumeric;
use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

//register acc
pub async fn email_register(email: String, password: String, sn: String, captcha_code: String) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let phone =  new_phone_who_dis();

    for _ in 0..3 {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json, text/javascript, */*; q=0.01"));
        headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("content-type", HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"));
        headers.insert("origin", HeaderValue::from_static("https://www.raccoongame.com"));
        headers.insert("priority", HeaderValue::from_static("u=1, i"));
        headers.insert("referer", HeaderValue::from_static("https://www.raccoongame.com/login?redirect_uri=https%3A%2F%2Fwww.raccoongame.com%2Fweb2%2Fdist%2F%23%2Fplatform%2Fcloudgame"));
        headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"150\", \"Google Chrome\";v=\"150\""));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("\"Linux\""));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36"));
        headers.insert("x-requested-with", HeaderValue::from_static("XMLHttpRequest"));

        let form_params = [
            ("email", email.as_str()),
            ("code", captcha_code.as_str()),
            ("password", password.as_str()),
            ("phone", phone.as_str()),
            ("country", "Myanmar"),
            ("sn", sn.as_str()),
            ("model", "Chrome/150.0.0.0"),
            ("version_code", "1"),
            ("version_name", "1.0.0"),
            ("device_name", "我的电脑"),
            ("os", "pc"),
        ];

        let response = client.post("https://www.raccoongame.com/users/emailRegister")
            .headers(headers)
            .form(&form_params)
            .send()
            .await;

        if let Ok(res) = response {
            if let Ok(response_json) = res.json::<Value>().await {
                if response_json["status"].as_i64() == Some(200) {
                    if let Some(returned_sn) = response_json["data"]["sn"].as_str() {
                        if returned_sn == sn {
                            if let Some(user_token) = response_json["data"]["user_token"].as_str() {
                                return Ok(user_token.to_string());
                            }
                        }
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    anyhow::bail!("Account registration failed or SN mismatch occurred across all attempts")
}

fn new_phone_who_dis() -> String {
    rand::rng()
        .sample_iter(rand::distr::uniform::Uniform::new_inclusive('0', '9').unwrap())
        .take(10)
        .collect()
}