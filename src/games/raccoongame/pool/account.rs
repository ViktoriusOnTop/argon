#[derive(serde::Serialize, serde::Deserialize)]
pub struct ACCOUNT{
    sn: Option<String>,
    token: Option<String>,
}

impl ACCOUNT{
    pub fn new(token: String, sn: String) -> Self{
        return Self{sn: Some(sn), token: Some(token)};
    }

    pub fn into_parts(self) -> (Option<String>, Option<String>) {
        (self.sn, self.token)
    }
}