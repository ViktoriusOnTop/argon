use crate::games::raccoongame::mail_account::mail_account::ACCOUNT;

//build full account
pub async fn build() -> anyhow::Result<(String, String)> {
    for _ in 0..3 {
        if let Ok(account) = ACCOUNT::new().await {
            if let Some(email) = account.email {
                return Ok((email, "some_token".to_string()));
            }
        }
    }
    anyhow::bail!("All steps failed three times, probably a upstream issue")
}