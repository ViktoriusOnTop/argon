use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub struct Rtcs {
    pub answer: String,
    pub candidates: Vec<String>,
}

const SIGNAL_DEADLINE_MS: u64 = 45_000;
const POST_ANSWER_GRACE_MS: u64 = 3_000;

pub async fn get_rtcs(
    ws_url: &str,
    ws_token: &str,
    sn: &str,
    gl_key: &str,
    sc_id: Value,
    play_config: Value,
    offer_sdp: &str,
) -> anyhow::Result<Rtcs> {
    crate::vlog!("[get_rtcs] inputs: url={} sn={} gl_key={} sc_id={}", ws_url, sn, gl_key, sc_id);

    let (ws, _) = connect_async(ws_url).await
        .map_err(|e| anyhow::anyhow!("signal ws connect failed: {}", e))?;
    let (mut write, mut read) = ws.split();

    let token = urlencoding::decode(ws_token)
        .map(|t| t.into_owned())
        .unwrap_or_else(|_| ws_token.to_string());

    write.send(Message::Text(json!({
        "id": "register",
        "type": "webUA",
        "uid": sn,
        "token": token,
    }).to_string().into())).await?;

    crate::vlog!("[get_rtcs] registered, waiting for ack");

    let mut ping = tokio::time::interval(Duration::from_secs(30));
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let started = std::cell::Cell::new(false);
    let mut answer: Option<String> = None;
    let mut candidates: Vec<String> = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_millis(SIGNAL_DEADLINE_MS);
    let mut answered_at: Option<tokio::time::Instant> = None;

    loop {
        if tokio::time::Instant::now() > deadline {
            anyhow::bail!("signal ws timed out after {}ms, raccoon left us on read", SIGNAL_DEADLINE_MS);
        }
        if let Some(t) = answered_at {
            if tokio::time::Instant::now() - t > Duration::from_millis(POST_ANSWER_GRACE_MS) {
                break;
            }
        }

        tokio::select! {
            _ = ping.tick() => {
                let _ = write.send(Message::Text(json!({
                    "id": "ping",
                    "uid": sn,
                    "type": "webUA",
                    "status": "gaming",
                    "sc_id": sc_id,
                }).to_string().into())).await;
            }
            msg = read.next() => {
                let Some(msg) = msg else {
                    anyhow::bail!("signal ws closed before we got an answer, rude");
                };
                let text = msg?.to_text()?.to_string();
                let data: Value = match serde_json::from_str(&text) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                crate::vlog!("[get_rtcs] output body: {}", data);

                match data["id"].as_str() {
                    Some("register_ack") => {
                        if data["code"].as_u64() == Some(200) && !started.get() {
                            started.set(true);
                            write.send(Message::Text(json!({
                                "id": "start_game",
                                "from": sn,
                                "to": gl_key,
                                "game_args": "",
                                "gp_num": 0,
                                "play_config": play_config,
                                "simpleHandler": Value::Null,
                                "body": {
                                    "force_soft_dec": 0,
                                    "session_id": sc_id,
                                    "sn_user_id": sn,
                                    "game_name": Value::Null,
                                    "joystick_num": 2,
                                },
                            }).to_string().into())).await?;
                        }
                    }
                    Some("start_game") => {
                        if data["from"].as_str() == Some(gl_key)
                            && data["body"]["code"].as_u64() == Some(200)
                        {
                            crate::vlog!("[get_rtcs] game ready, sending rtc offer");
                            write.send(Message::Text(json!({
                                "id": "rtc_sdp",
                                "from": sn,
                                "to": gl_key,
                                "body": { "sdp": offer_sdp, "type": "offer" },
                            }).to_string().into())).await?;
                        }
                    }
                    Some("rtc_sdp") => {
                        let body = &data["body"];
                        match body["type"].as_str() {
                            Some("answer") => {
                                answer = Some(body["sdp"].to_string());
                                answered_at = Some(tokio::time::Instant::now());
                            }
                            Some("candidate") => {
                                if let Some(c) = body["sdp"].as_str() {
                                    candidates.push(c.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let Some(answer) = answer else {
        anyhow::bail!("no sdp answer came back, raccoon is playing hard to get");
    };
    crate::vlog!("[get_rtcs] got answer with {} candidates", candidates.len());
    Ok(Rtcs { answer, candidates })
}

