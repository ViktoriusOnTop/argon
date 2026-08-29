use std::sync::LazyLock;
use regex::Regex;
use serde_json::Value;
use crate::games::raccoongame::limiter::MAIL_TM_LIMITER;

static CAPTCHA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"【(.*?)】").unwrap()
});

pub async fn fetch_captcha(token: String) -> anyhow::Result<String> {
    for _ in 0..3 {
        if let Ok(email) = fetch_email(token.clone()).await {
            if let Some(members) = email["hydra:member"].as_array() {
                if !members.is_empty() {
                    if let Some(intro) = members[0]["intro"].as_str() {
                        if let Ok(captcha) = extract_captcha(intro) {
                            return Ok(captcha);
                        }
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Captcha fetch failed");
}

fn extract_captcha(message: &str) -> anyhow::Result<String> {
    if let Some(captures) = CAPTCHA_REGEX.captures(message) {
        if let Some(matched_code) = captures.get(1) {
            return Ok(matched_code.as_str().trim().to_string());
        }
    }
    anyhow::bail!("Captcha pattern matching failed")
}

async fn fetch_email(token: String) -> anyhow::Result<Value> {
    for _ in 0..3 {
        let client = reqwest::Client::new();
        MAIL_TM_LIMITER.until_ready().await;

        let response = client.get("https://api.mail.tm/messages?page=1")
            .bearer_auth(&token)
            .send()
            .await;

        if let Ok(res) = response {
            if let Ok(email_json) = res.json::<Value>().await {
                if let Some(total) = email_json["hydra:totalItems"].as_i64() {
                    if total > 0 {
                        return Ok(email_json);
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Email fetch failed or inbox empty");
}
