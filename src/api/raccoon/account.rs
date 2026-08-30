// GET /api/raccoon/account

use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use crate::games::raccoongame::raccoon_account::full_account_builder::build;

#[derive(Serialize)]
pub struct AccountResponse {
    sn: String,
    token: String,
}

pub async fn get_account() -> anyhow::Result<Json<AccountResponse>, StatusCode> {
    let (token, sn) = build().await.unwrap();

    Ok(Json(AccountResponse { sn, token }))
}