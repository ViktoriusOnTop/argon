
use crate::games::raccoongame::mail_account::{mail_gw, mail_tm};

//todo: add more mail providers
//holds accounts (handler for email acc creation)
pub struct ACCOUNT{
    pub mail_tm_token: Option<String>,
    pub mail_gw_token: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>, //redundancy for more providers
}

impl ACCOUNT{
    pub async fn new() -> anyhow::Result<ACCOUNT>{
        for _ in 0..3 {
            if let Ok((email, token)) = mail_tm::make_email::make_email().await{
                return Ok(ACCOUNT{mail_tm_token: Option::from(token), mail_gw_token: None, email: Some(email), password: None})
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
        for _ in 0..3 {
            if let Ok((email, token)) = mail_gw::make_email::make_email().await{
                return Ok(ACCOUNT{mail_tm_token: None, mail_gw_token: Option::from(token), email: Some(email), password: None})
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        }
        anyhow::bail!("Mail account could not be created")
    }
    pub async fn get_captcha(&self) -> anyhow::Result<String>{
        if let Some(token) = &self.mail_tm_token{
            for _ in 0..3 {
                if let Ok(captcha) = mail_tm::fetch_emails::fetch_captcha(token.clone()).await{
                    return Ok(captcha);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
        if let Some(token) = &self.mail_gw_token{
            for _ in 0..3 {
                if let Ok(captcha) = mail_gw::fetch_emails::fetch_captcha(token.clone()).await{
                    return Ok(captcha);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }
        anyhow::bail!("Account::new failed, and still called get_captcha")
    }
}