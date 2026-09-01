use rand::distr::Alphanumeric;
use rand::RngExt;
use crate::games::raccoongame::mail_account::mail_account::ACCOUNT;
use crate::games::raccoongame::raccoon_account::register_account::email_register;
use crate::games::raccoongame::raccoon_account::send_email::send_email;

pub async fn build() -> anyhow::Result<(String, String, String, String)> {
    let mut last_err = String::from("no attempt succeeded");
    for _ in 0..3 {
        match ACCOUNT::new().await {
            Err(e) => last_err = format!("make_account: {e:#}"),
            Ok(account) => {
                if let Some(email) = &account.email {
                    let sn = generate_sn();
                    let pass = generate_string(15);

                    match send_email(email.clone(), sn.clone()).await {
                        Err(e) => last_err = format!("send_email: {e:#}"),
                        Ok(_) => match account.get_captcha().await {
                            Err(e) => last_err = format!("get_captcha: {e:#}"),
                            Ok(captcha_code) => match email_register(email.clone(), pass.clone(), sn.clone(), captcha_code).await {
                                Ok(user_token) => return Ok((user_token, sn, email.clone(), pass.clone())),
                                Err(e) => last_err = format!("email_register: {e:#}"),
                            },
                        },
                    }
                } else {
                    last_err = "make_account: no email in response".to_string();
                }
            }
        }
        if crate::games::raccoongame::limiter::RETRY_BACKOFF_ENABLED {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
    }
    anyhow::bail!("build failed: {}", last_err)
}

fn generate_sn() -> String {
    const CHARSET: &[u8] = b"0123456789abcdef";
    let mut rng = rand::rng();

    (0..32)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
