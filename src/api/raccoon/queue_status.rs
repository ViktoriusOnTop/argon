// GET /api/raccoon/queue
use axum::extract::Query;
use axum::http::{header, HeaderName, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::games::raccoongame::play::full_gameplay::finish_rtc;
use crate::logs::write::raccoon::database::accountdb::db::account_db;

#[derive(Deserialize)]
pub struct QueueParams {
    pub public_id: String,
}

#[derive(Serialize)]
pub struct QueueResponse {
    pub status: String,
    pub position: u64,
    pub answer: String,
    pub candidates: Vec<String>,
}

pub async fn queue_status(Query(params): Query<QueueParams>) -> Result<Json<QueueResponse>, (StatusCode, [(HeaderName, &'static str); 1])> {
    crate::vlog!("[queue_status] inputs: public_id={}", params.public_id);

    let db = account_db().map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    let Some(session) = db.get_session(&params.public_id).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, [(header::RETRY_AFTER, "10")]))? else {
        return Err((StatusCode::NOT_FOUND, [(header::RETRY_AFTER, "10")]));
    };

    let Some(active) = db.active(&params.public_id).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, [(header::RETRY_AFTER, "10")]))? else {
        return Err((StatusCode::NOT_FOUND, [(header::RETRY_AFTER, "10")]));
    };

    match finish_rtc(&active.token, &active.sn, &session.game_key, &session.queue_id, &session.offer_sdp).await {
        Ok(crate::games::raccoongame::play::full_gameplay::PlayFinish::StillQueued { position }) => {
            crate::vlog!("[queue_status] output: public_id={} still queued at {}", params.public_id, position);
            Ok(Json(QueueResponse { status: "queue".to_string(), position, answer: String::new(), candidates: Vec::new() }))
        }
        Ok(crate::games::raccoongame::play::full_gameplay::PlayFinish::Ready(rtcs)) => {
            if let Err(e) = db.delete_session(&params.public_id) {
                eprintln!("delete_session falled for {}: {}", params.public_id, e);
            }
            if let Err(e) = db.consume(&params.public_id) {
                eprintln!("consume falled for {}: {}", params.public_id, e);
            }
            crate::vlog!("[queue_status] output: ready for public_id={}, answer + {} candidates", params.public_id, rtcs.candidates.len());
            Ok(Json(QueueResponse { status: "ready".to_string(), position: 0, answer: rtcs.answer, candidates: rtcs.candidates }))
        }
        Err(e) => {
            eprintln!("finish_rtc falled for public_id {}: {}", params.public_id, e);
            Err((StatusCode::BAD_GATEWAY, [(header::RETRY_AFTER, "10")]))
        }
    }
}
