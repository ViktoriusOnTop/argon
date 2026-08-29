use rand::distr::Alphanumeric;
use rand::RngExt;
use serde_json::{json, Value};
use crate::games::raccoongame::limiter::MAIL_TM_LIMITER;

pub async fn make_email() -> anyhow::Result<(String, String)> {
    let domain = get_domain().await?;
    #[cfg(debug_assertions)]
    println!("{}", domain);
    let email_start = generate_string(15);
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
    for _ in 0..3{
        MAIL_TM_LIMITER.until_ready().await;
        let domain_json: Value = reqwest::get("https://api.mail.tm/domains")
            .await?
            .json()
            .await?;
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
    for _ in 0..3{
        let client = reqwest::Client::new();

        MAIL_TM_LIMITER.until_ready().await;
        let response_json: Value = client.post("https://api.mail.tm/accounts")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let recieved_email = response_json["address"].as_str().unwrap();

        if recieved_email == email{
            let id = response_json["id"].as_str().unwrap().to_string();
            return Ok(id);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Account creation failed");
}

async fn get_token(id: String, body: Value) -> anyhow::Result<String> {
    for _ in 0..3{
        let client = reqwest::Client::new();
    
        MAIL_TM_LIMITER.until_ready().await;
        let token_json: Value = client.post("https://api.mail.tm/token")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if token_json["id"] == id{
            return Ok(token_json["token"].as_str().unwrap().to_string());
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("Token creation failed");
}