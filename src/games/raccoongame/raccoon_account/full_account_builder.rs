use rand::distr::Alphanumeric;
use rand::RngExt;
use crate::games::raccoongame::mail_account::mail_account::ACCOUNT;
use crate::games::raccoongame::raccoon_account::register_account::email_register;
use crate::games::raccoongame::raccoon_account::send_email::send_email;

pub async fn build() -> anyhow::Result<(String, String)> {
    for _ in 0..3 {
        if let Ok(account) = ACCOUNT::new().await {
            if let Some(email) = &account.email {
                let sn = generate_sn();
                let pass = generate_string(15);

                if send_email(email.clone(), sn.clone()).await.is_ok() {

                    if let Ok(captcha_code) = account.get_captcha().await {
                        if let Ok(user_token) = email_register(email.clone(), pass.clone(), sn.clone(), captcha_code).await {
                            return Ok((user_token, sn));
                        }
                    }
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    anyhow::bail!("All steps failed three times probably cut off by raccoon")
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
