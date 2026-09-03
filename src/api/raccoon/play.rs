use axum::http::{header, HeaderName, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::games::raccoongame::play::full_gameplay::get_rtc;
use crate::logs::write::raccoon::database::accountdb::db::{account_db, PlaySession};

#[derive(Deserialize)]
pub struct PlayRequest {
    pub public_id: String,
    pub game_key: String,
    pub offer_sdp: String,
}

#[derive(Serialize)]
pub struct PlayResponse {
    pub status: String,
    pub position: u64,
    pub answer: String,
    pub candidates: Vec<String>,
}

pub async fn play(Json(req): Json<PlayRequest>) -> Result<Json<PlayResponse>, (StatusCode, [(HeaderName, &'static str); 1])> {
    crate::vlog!("[play] inputs: public_id={} game_key={}", req.public_id, req.game_key);

    let db = account_db().map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, [(header::RETRY_AFTER, "30")]))?;

    let probed = db.games_count().unwrap_or(0) > 0;
    if probed && !db.is_playable(&req.game_key).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, [(header::RETRY_AFTER, "30")]))? {
        crate::vlog!("[play] blocked: public_id={} game_key={} not in playable catalog", req.public_id, req.game_key);
        return Err((StatusCode::FORBIDDEN, [(header::RETRY_AFTER, "30")]));
    }

    let Some(active) = db.active(&req.public_id).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, [(header::RETRY_AFTER, "30")]))? else {
        return Err((StatusCode::NOT_FOUND, [(header::RETRY_AFTER, "30")]));
    };

    match get_rtc(&active.token, &active.sn, &req.game_key, &req.offer_sdp).await {
        Ok(crate::games::raccoongame::play::full_gameplay::PlayStart::Ready(rtcs)) => {
            if let Err(e) = db.consume(&req.public_id) {
                eprintln!("consume falled for {}: {}", req.public_id, e);
            }
            crate::vlog!("[play] output: ready for public_id={}, answer + {} candidates", req.public_id, rtcs.candidates.len());
            Ok(Json(PlayResponse { status: "ready".to_string(), position: 0, answer: rtcs.answer, candidates: rtcs.candidates }))
        }
        Ok(crate::games::raccoongame::play::full_gameplay::PlayStart::Queued { queue_id, position }) => {
            let session = PlaySession { game_key: req.game_key.clone(), offer_sdp: req.offer_sdp.clone(), queue_id, created_at: 0 };
            if let Err(e) = db.put_session(&req.public_id, &session) {
                eprintln!("put_session falled for {}: {}", req.public_id, e);
            }
            crate::vlog!("[play] output: queued for public_id={} at {}", req.public_id, position);
            Ok(Json(PlayResponse { status: "queue".to_string(), position, answer: String::new(), candidates: Vec::new() }))
        }
        Err(e) => {
            eprintln!("get_rtc falled for public_id {}: {}", req.public_id, e);
            Err((StatusCode::BAD_GATEWAY, [(header::RETRY_AFTER, "30")]))
        }
    }
}
