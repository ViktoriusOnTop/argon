use reqwest::get;
use serde_json::Value;
use crate::games::raccoongame::limiter::MAIL_GW_LIMITER;

async fn fetch_captcha(token: String) -> anyhow::Result<String> {
    for _ in 0..3{
        let email = fetch_email(token.clone()).await?;
        let captcha_message: String = email["hydra:member"][0]["intro"].as_str().unwrap().to_string();
        if let Ok(captcha) = extract_captcha(captcha_message) {
            if captcha.chars().count() == 6 {
                return Ok(captcha);
            }
        }
    }
    anyhow::bail!("Captcha fetch failed");
}

fn extract_captcha(message: String) -> anyhow::Result<String> {
    let (_, right) = message.split_once('【').unwrap();
    let (code_uncomplete, _) = right.split_once('】').unwrap();
    let code = code_uncomplete.trim();
    Ok(code.to_string())
}

async fn fetch_email(token: String) -> anyhow::Result<Value>{
    for _ in 0..3{
        let client = reqwest::Client::new();

        MAIL_GW_LIMITER.until_ready().await;
        let email_json: Value = client.get("https://api.mail.gw/messages?page=1")
            .bearer_auth(&token)
            .send()
            .await?
            .json()
            .await?;

        let exists = email_json["hydra:totalItems"].as_i64().unwrap() >= 0;

        if exists{
            return Ok(email_json);
        }
    }
    anyhow::bail!("Email fetch failed");
}