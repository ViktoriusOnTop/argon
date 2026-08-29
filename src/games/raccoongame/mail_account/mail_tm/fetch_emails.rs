use serde_json::Value;
use crate::games::raccoongame::limiter::MAIL_TM_LIMITER;

pub async fn fetch_captcha(token: String) -> anyhow::Result<String> {
    for _ in 0..3 {
        if let Ok(email) = fetch_email(token.clone()).await {
            if let Some(members) = email["hydra:member"].as_array() {
                if !members.is_empty() {
                    if let Some(intro) = members[0]["intro"].as_str() {
                        if let Ok(captcha) = extract_captcha(intro.to_string()) {
                            if captcha.chars().count() == 6 {
                                return Ok(captcha);
                            }
                        }
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Captcha fetch failed");
}

fn extract_captcha(message: String) -> anyhow::Result<String> {
    if let Some((_, right)) = message.split_once('【') {
        if let Some((code_uncomplete, _)) = right.split_once('】') {
            return Ok(code_uncomplete.trim().to_string());
        }
    }
    anyhow::bail!("Captcha markers not found in message")
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
