use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::get_client;
#[derive(Deserialize)]
pub struct PooPooPeePeeParams {
    pub q: String,
}

#[derive(Serialize)]
pub struct PooPooPeePeeResponse {
    pub answer: Value,
}

pub async fn search(
    Query(params): Query<PooPooPeePeeParams>,
) -> Result<Json<PooPooPeePeeResponse>, (StatusCode, [(axum::http::HeaderName, &'static str); 1], Json<Value>)> {
    let client_cell = get_client();

    let Some(client) = client_cell.get() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            [(axum::http::header::RETRY_AFTER, "300")],
            Json(serde_json::json!({"error": "search unavailable, argon is running without docker or meilisearch isnt up yet"})),
        ));
    };

    let search_result = crate::games::raccoongame::searching::meilisearch::search_meilisearch(
        client,
        &params.q
    )
        .await
        .map_err(|e| {
            eprintln!("mili mommy search go boom no no :( {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(axum::http::header::RETRY_AFTER, "5")],
                Json(serde_json::json!({"error": "search failed"})),
            )
        })?;

    Ok(Json(PooPooPeePeeResponse {
        answer: serde_json::json!(search_result),
    }))
}
