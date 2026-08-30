use rand::distr::Alphanumeric;
use rand::RngExt;
use serde_json::{json, Value};
use std::time::Duration;
use crate::games::raccoongame::limiter::MAIL_GW_LIMITER;

pub async fn make_email() -> anyhow::Result<(String, String)> {
    let domain = get_domain().await?;
    let email_start = generate_string(15).to_lowercase();
    let pass = generate_string(15);

    let email = format!("{}@{}", email_start, domain);

    let body = json!({
        "address": email,
        "password": pass,
        });

    let id = make_account(&email, &body).await?;

    let token = get_token(id, body).await?;

    Ok((email, token))
}


async fn get_domain() -> anyhow::Result<String> {
    for attempt in 0..3{
        crate::dlog!("[mail_gw] inputs: url=https://api.mail.gw/domains attempt={}/3", attempt + 1);
        MAIL_GW_LIMITER.until_ready().await;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        let domain_json: Value = client.get("https://api.mail.gw/domains")
            .send()
            .await?
            .json()
            .await?;
        crate::dlog!("[mail_gw] output body: {}", domain_json);
        let domain = domain_json["hydra:member"][0]["domain"]
            .as_str();
        if let Some(domain) = domain{
            return Ok(domain.to_string());
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Mail domain not found");
}

fn generate_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

async fn make_account(email: &String, body: &Value) -> anyhow::Result<String> {
    for attempt in 0..3{
        crate::dlog!("[mail_gw] inputs: url=https://api.mail.gw/accounts attempt={}/3 body={}", attempt + 1, body);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        MAIL_GW_LIMITER.until_ready().await;
        let response_json: Value = client.post("https://api.mail.gw/accounts")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        crate::dlog!("[mail_gw] output body: {}", response_json);

        let recieved_email = response_json["address"].as_str();
        if let Some(_recieved_email) = recieved_email{
            let recieved_email = response_json["address"].as_str().unwrap();
            if recieved_email == email{
                if let Some(received_email) = response_json["address"].as_str() {
                    if received_email == email {
                        if let Some(id_str) = response_json["id"].as_str() {
                            return Ok(id_str.to_string());
                        }
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Account creation failed");
}

async fn get_token(id: String, body: Value) -> anyhow::Result<String> {
    for attempt in 0..3{
        crate::dlog!("[mail_gw] inputs: url=https://api.mail.gw/token attempt={}/3 id={} body={}", attempt + 1, id, body);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        MAIL_GW_LIMITER.until_ready().await;
        let token_json: Value = client.post("https://api.mail.gw/token")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        crate::dlog!("[mail_gw] output body: {}", token_json);

        if token_json["id"] == id {
            if let Some(token_str) = token_json["token"].as_str() {
                return Ok(token_str.to_string());
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Token creation failed");
}