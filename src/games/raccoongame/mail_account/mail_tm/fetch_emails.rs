use std::sync::LazyLock;
use regex::Regex;
use serde_json::Value;
use crate::games::raccoongame::limiter::MAIL_TM_LIMITER;

//regex for getting captcha is it better than splitting idk
static CAPTCHA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"【(.*?)】").unwrap()
});

//buys a $1 billion private jet
pub async fn fetch_captcha(token: String) -> anyhow::Result<String> {
    for attempt in 0..3 {
        crate::vlog!("[mail_tm] fetch_captcha inputs: token={} attempt={}/3", token, attempt + 1);
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
        if crate::games::raccoongame::limiter::RETRY_BACKOFF_ENABLED {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    }
    anyhow::bail!("Captcha fetch failed");
}

//extracts the captcha if you need this comment then your just dumb sorry but its true
fn extract_captcha(message: &str) -> anyhow::Result<String> {
    if let Some(captures) = CAPTCHA_REGEX.captures(message) {
        if let Some(matched_code) = captures.get(1) {
            return Ok(matched_code.as_str().trim().to_string());
        }
    }
    anyhow::bail!("Captcha pattern matching failed")
}

//fetch email kinda obvious
async fn fetch_email(token: String) -> anyhow::Result<Value> {
    for attempt in 0..3 {
        crate::vlog!("[mail_tm] inputs: url=https://api.mail.tm/messages?page=1 attempt={}/3 bearer_token={}", attempt + 1, token);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        MAIL_TM_LIMITER.until_ready().await;

        let response = client.get("https://api.mail.tm/messages?page=1")
            .bearer_auth(&token)
            .send()
            .await;
        match &response {
            Ok(res) => crate::vlog!("[mail_tm] output: http status={}", res.status()),
            Err(e) => crate::vlog!("[mail_tm] output: reqwest errored: {}", e),
        }

        if let Ok(res) = response {
            if let Ok(email_json) = res.json::<Value>().await {
                crate::vlog!("[mail_tm] output body: {}", email_json);
                if let Some(total) = email_json["hydra:totalItems"].as_i64() {
                    if total > 0 {
                        return Ok(email_json);
                    }
                }
            }
        }
        if crate::games::raccoongame::limiter::RETRY_BACKOFF_ENABLED {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    }
    anyhow::bail!("Email fetch failed or inbox empty");
}
