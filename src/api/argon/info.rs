use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfoResponse {
    pub version: String,
    pub features: Vec<String>,
    pub repo: String,
    pub contributors: Vec<String>,
}

pub async fn get_info() -> Json<InfoResponse> {
    let info = InfoResponse {
        version: "0.9.9".to_string(),
        features: vec![
            "mail endpoints work".to_string(),
            "raccoon registration".to_string(),
            "search through games".to_string(),
            "gameplay works".to_string(),
            "v1 out!".to_string(),
        ],
        repo: "https://github.com/ViktoriusOnTop/argon".to_string(),
        contributors: vec!["ViktoriusOnTop".to_string()],
    };

    Json(info)
}