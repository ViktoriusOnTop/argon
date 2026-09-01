#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ACCOUNT{
    sn: Option<String>,
    token: Option<String>,
    email: Option<String>,
    password: Option<String>,
    public_id: Option<String>,
}

impl ACCOUNT{
    pub fn new(token: String, sn: String, email: String, password: String, public_id: String) -> Self{
        return Self{sn: Some(sn), token: Some(token), email: Some(email), password: Some(password), public_id: Some(public_id)};
    }

    pub fn stub(public_id: String) -> Self {
        return Self{sn: None, token: None, email: None, password: None, public_id: Some(public_id)};
    }

    pub fn public_id(&self) -> Option<&str> {
        self.public_id.as_deref()
    }

    pub fn into_parts(self) -> (Option<String>, Option<String>) {
        (self.sn, self.token)
    }
}