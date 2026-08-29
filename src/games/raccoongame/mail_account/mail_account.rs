
use crate::games::raccoongame::mail_account::{mail_gw, mail_tm};

//todo: add more mail providers
struct ACCOUNT{
    mail_tm_token: Option<String>,
    mail_gw_token: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl ACCOUNT{
    async fn new() -> anyhow::Result<ACCOUNT>{
        for _ in 0..3 {
            if let Ok((email, token)) = mail_tm::make_email::make_email().await{
                return Ok(ACCOUNT{mail_tm_token: Option::from(token), mail_gw_token: None, email: Some(email), password: None})
            }
        }
        for _ in 0..3 {
            if let Ok((email, token)) = mail_gw::make_email::make_email().await{
                return Ok(ACCOUNT{mail_tm_token: None, mail_gw_token: Option::from(token), email: Some(email), password: None})
            }
        }
        anyhow::bail!("Mail account could not be created")
    }
    async fn get_captcha(&self) -> anyhow::Result<String>{
        if let Some(token) = &self.mail_tm_token{
            for _ in 0..3 {
                if let Ok(captcha) = mail_tm::fetch_emails::fetch_captcha(token.clone()).await{
                    return Ok(captcha);
                }
            }
        }
        if let Some(token) = &self.mail_gw_token{
            for _ in 0..3 {
                if let Ok(captcha) = mail_gw::fetch_emails::fetch_captcha(token.clone()).await{
                    return Ok(captcha);
                }
            }
        }
        anyhow::bail!("Account::new failed, and still called get_captcha")
    }
}