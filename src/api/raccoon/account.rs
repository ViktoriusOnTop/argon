// GET /api/raccoon/account

use axum::http::{header, HeaderName, StatusCode};
use axum::Json;
use crate::get_pool;
use crate::logs::write::raccoon::database::accountdb::db::account_db;
use crate::logs::write::raccoon::database::accountdb::account::PublicAccount;

pub async fn get_account() -> Result<Json<PublicAccount>, (StatusCode, [(HeaderName, &'static str); 1])> {
    let cell = get_pool();
    let pool = cell.get().ok_or((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    let account = pool.get_account().await
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    let Some(public_id) = account.public_id().map(str::to_string) else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]));
    };

    let db = account_db().map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    crate::vlog!("[get_account] inputs: public_id={}", public_id);

    match db.pop_account(&public_id) {
        Ok(Some(public)) => {
            crate::vlog!("[get_account] output: public_id={}", public.public_id);
            Ok(Json(public))
        }
        Ok(None) => Err((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")])),
        Err(e) => {
            eprintln!("pop_account falled for {}: {}", public_id, e);
            Err((StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))
        }
    }
}
