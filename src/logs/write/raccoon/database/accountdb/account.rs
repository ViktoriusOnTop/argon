use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ACCOUNT {
    pub sn: String,
    pub token: String,
    pub email: String,
    pub password: String,
    pub public_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicAccount {
    pub public_id: String,
}

pub struct AccountThatsPublic {
    pub public_id: String,
}

impl From<&ACCOUNT> for PublicAccount {
    fn from(a: &ACCOUNT) -> Self {
        Self {
            public_id: a.public_id.clone(),
        }
    }
}
