// GET /api/raccoon/account

use axum::http::{header, HeaderName, StatusCode};
use axum::Json;
use serde::Serialize;
use crate::get_pool;

#[derive(Serialize)]
pub struct AccountResponse {
    sn: Option<String>,
    token: Option<String>,
}

pub async fn get_account() -> Result<Json<AccountResponse>, (StatusCode, [(HeaderName, &'static str); 1])> {
    let cell = get_pool();
    let pool = cell.get().ok_or((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    match pool.get_account().await {
        Some(account) => {
            let (sn, token) = account.into_parts();
            Ok(Json(AccountResponse { sn, token }))
        }
        None => Err((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")])),
    }
}