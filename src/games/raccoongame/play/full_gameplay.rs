use base64::Engine;
use serde_json::Value;

use crate::games::raccoongame::play::check_cost::check_cost;
use crate::games::raccoongame::play::get_pos::get_pos;
use crate::games::raccoongame::play::get_rtcs::{get_rtcs, Rtcs};
use crate::games::raccoongame::play::play_game::play_game;

use aes::Aes256;
use aes::cipher::{BlockDecrypt, KeyInit};

const RESULT_KEY: &[u8; 32] = b"fd39e724f7c1e4b3d34bc7c72b5349c3";
const RESULT_IV: &[u8; 16] = b"dd39e4a3337fe25a";


fn decrypt_result(result_b64: &str) -> anyhow::Result<Value> {
    let raw = base64::engine::general_purpose::STANDARD.decode(result_b64)?;
    if raw.is_empty() || raw.len() % 16 != 0 {
        anyhow::bail!("result blob is {} bytes, not a multiple of 16", raw.len());
    }

    let cipher = Aes256::new(RESULT_KEY.into());
    let mut out = Vec::with_capacity(raw.len());
    let mut prev: [u8; 16] = *RESULT_IV;
    for chunk in raw.chunks_exact(16) {
        let mut block: [u8; 16] = chunk.try_into()?;
        cipher.decrypt_block((&mut block).into());
        for (b, p) in block.iter_mut().zip(prev.iter()) {
            *b ^= p;
        }
        prev = chunk.try_into()?;
        out.extend_from_slice(&block);
    }

    let pad = *out.last().ok_or_else(|| anyhow::anyhow!("result blob empty"))? as usize;
    if pad == 0 || pad > 16 || pad > out.len() {
        anyhow::bail!("result blob has bogus pkcs7 padding byte {}", pad);
    }
    out.truncate(out.len() - pad);

    let parsed = serde_json::from_slice::<Value>(&out)?;
    if !parsed.is_object() {
        anyhow::bail!("decrypted result is not an object, raccoon sent soup");
    }
    Ok(parsed)
}

pub enum PlayStart {
    Ready(Rtcs),
    Queued { queue_id: String, position: u64 },
}

pub enum PlayFinish {
    StillQueued { position: u64 },
    Ready(Rtcs),
}

fn as_id(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.to_string())
        .or_else(|| v.as_u64().map(|n| n.to_string()))
}

fn parse_server_data(server_data: Value) -> anyhow::Result<(String, Value, String, Value, String, String)> {
    let sc_id = as_id(&server_data["sc_id"])
        .or_else(|| as_id(&server_data["play_id"]))
        .ok_or_else(|| anyhow::anyhow!("server_data missing sc_id, wtf raccoon"))?;
    let bs_sc_id = server_data["bs_sc_id"]
        .as_u64()
        .map(Value::from)
        .or_else(|| server_data["bs_sc_id"].as_str().map(Value::from))
        .unwrap_or_else(|| Value::from(sc_id.clone()));
    let gl_key = server_data["gl_key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing gl_key, no game node assigned"))?
        .to_string();
    let play_config = server_data["play_config"].clone();
    let ws_url = server_data["message_server"]["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing message_server.url, no signaling link"))?
        .to_string();
    let ws_token = server_data["message_server"]["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing message_server.token"))?
        .to_string();
    Ok((sc_id, bs_sc_id, gl_key, play_config, ws_url, ws_token))
}

pub async fn get_rtc(
    token: &str,
    sn: &str,
    game_key: &str,
    offer_sdp: &str,
) -> anyhow::Result<PlayStart> {
    crate::vlog!("[get_rtc] inputs: sn={} game_key={}", sn, game_key);

    let _ = check_cost(token, sn, game_key).await?;

    let play = play_game(token, sn, game_key, None).await?;

    if play["status"].as_u64() == Some(201)
        || (play["status"].as_u64() == Some(200) && play["data"]["play_queue_id"].is_string())
        || (play["status"].as_u64() == Some(201) && play["data"]["play_queue_id"].is_number())
        || (play["status"].as_u64() == Some(200) && play["data"]["play_queue_id"].is_number())
    {
        let queue_id = as_id(&play["data"]["play_queue_id"])
            .ok_or_else(|| anyhow::anyhow!("got queued but no queue id came with it"))?;
        let position = play["data"]["queue_pos"].as_u64().unwrap_or(1);
        crate::vlog!("[get_rtc] queued under {} at {}", queue_id, position);
        return Ok(PlayStart::Queued { queue_id, position });
    }

    let result = play["data"]["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("playGame had no result, status {}", play["status"]))?;
    let server_data = decrypt_result(result)?;
    let (_sc_id, bs_sc_id, gl_key, play_config, ws_url, ws_token) = parse_server_data(server_data)?;

    crate::vlog!("[get_rtc] server ready sc_id={} gl_key={} ws={}", bs_sc_id, gl_key, ws_url);

    let rtcs = get_rtcs(&ws_url, &ws_token, sn, &gl_key, bs_sc_id, play_config, offer_sdp).await?;
    crate::vlog!("[get_rtc] done, answer + {} candidates", rtcs.candidates.len());
    Ok(PlayStart::Ready(rtcs))
}

pub async fn finish_rtc(
    token: &str,
    sn: &str,
    game_key: &str,
    queue_id: &str,
    offer_sdp: &str,
) -> anyhow::Result<PlayFinish> {
    crate::vlog!("[finish_rtc] inputs: sn={} game_key={} queue_id={}", sn, game_key, queue_id);

    let pos = get_pos(token, sn, queue_id).await?;
    if pos != 0 {
        crate::vlog!("[finish_rtc] queue pos {}", pos);
        return Ok(PlayFinish::StillQueued { position: pos });
    }

    let claim = play_game(token, sn, game_key, Some(queue_id)).await?;
    let result = claim["data"]["result"].as_str().ok_or_else(|| {
        anyhow::anyhow!("queue claim had no result, claim status {}", claim["status"])
    })?;
    let server_data = decrypt_result(result)?;
    let (_sc_id, bs_sc_id, gl_key, play_config, ws_url, ws_token) = parse_server_data(server_data)?;

    crate::vlog!("[finish_rtc] server ready sc_id={} gl_key={} ws={}", bs_sc_id, gl_key, ws_url);

    let rtcs = get_rtcs(&ws_url, &ws_token, sn, &gl_key, bs_sc_id, play_config, offer_sdp).await?;
    crate::vlog!("[finish_rtc] done, answer + {} candidates", rtcs.candidates.len());
    Ok(PlayFinish::Ready(rtcs))
}

#[cfg(test)]
mod e2e {
    use super::{check_cost, get_pos, get_rtcs, parse_server_data, play_game};

    const GAME_KEY: &str = "kj0529";
    const MAX_POLLS: u32 = 120;
    const POLL_SECS: u64 = 5;

    fn recvonly_offer() -> String {
        let fp = "AB:CD:EF:12:34:56:78:9A:BC:DE:F0:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:12:34:56:78:9A";
        let mut s = String::new();
        s.push_str("v=0\r\n");
        s.push_str("o=- 4611731400430051336 2 IN IP4 127.0.0.1\r\n");
        s.push_str("s=-\r\n");
        s.push_str("t=0 0\r\n");
        s.push_str("a=group:BUNDLE 0 1\r\n");
        s.push_str("a=extmap-allow-mixed\r\n");
        s.push_str("a=msid-semantic: WMS\r\n");
        s.push_str("m=audio 9 UDP/TLS/RTP/SAVPF 111\r\n");
        s.push_str("c=IN IP4 0.0.0.0\r\n");
        s.push_str("a=rtcp:9 IN IP4 0.0.0.0\r\n");
        s.push_str("a=ice-ufrag:4ZcD\r\n");
        s.push_str("a=ice-pwd:2/1muCWoOi3uLifh0NuRHlUL\r\n");
        s.push_str("a=ice-options:trickle\r\n");
        s.push_str(&format!("a=fingerprint:sha-256 {}\r\n", fp));
        s.push_str("a=setup:actpass\r\n");
        s.push_str("a=mid:0\r\n");
        s.push_str("a=extmap:1 urn:ietf:params:rtp-hdrext:ssrc-audio-level\r\n");
        s.push_str("a=recvonly\r\n");
        s.push_str("a=rtcp-mux\r\n");
        s.push_str("a=rtpmap:111 opus/48000/2\r\n");
        s.push_str("a=fmtp:111 minptime=10;useinbandfec=1\r\n");
        s.push_str("m=video 9 UDP/TLS/RTP/SAVPF 96 97\r\n");
        s.push_str("c=IN IP4 0.0.0.0\r\n");
        s.push_str("a=rtcp:9 IN IP4 0.0.0.0\r\n");
        s.push_str("a=ice-ufrag:4ZcD\r\n");
        s.push_str("a=ice-pwd:2/1muCWoOi3uLifh0NuRHlUL\r\n");
        s.push_str("a=ice-options:trickle\r\n");
        s.push_str(&format!("a=fingerprint:sha-256 {}\r\n", fp));
        s.push_str("a=setup:actpass\r\n");
        s.push_str("a=mid:1\r\n");
        s.push_str("a=recvonly\r\n");
        s.push_str("a=rtcp-mux\r\n");
        s.push_str("a=rtcp-rsize\r\n");
        s.push_str("a=rtpmap:96 H264/90000\r\n");
        s.push_str("a=fmtp:96 level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f\r\n");
        s.push_str("a=rtcp-fb:96 nack\r\n");
        s.push_str("a=rtcp-fb:96 nack pli\r\n");
        s.push_str("a=rtcp-fb:96 goog-remb\r\n");
        s.push_str("a=rtpmap:97 rtx/90000\r\n");
        s.push_str("a=fmtp:97 apt=96\r\n");
        s
    }

    #[tokio::test]
    async fn full_chain() {
        let offer = recvonly_offer();
        let db = crate::logs::write::raccoon::database::accountdb::db::account_db().expect("open db");

        let accounts = db.all_accounts().expect("read");
        if accounts.is_empty() {
            panic!("no accounts in pool, run the server first so workers stock it");
        }

        let mut last_status = 0;
        for a in &accounts {
            let play = play_game(&a.token, &a.sn, GAME_KEY, None).await.expect("playGame failed");
            last_status = play["status"].as_u64().unwrap_or(0);
            println!("trying sn={} -> status={}", &a.sn[..8], last_status);
            if last_status == 200 || last_status == 201 {
                return run_chain(a, &offer, play).await;
            }
        }
        panic!("no usable account in pool, last status={}", last_status);
    }

    async fn run_chain(a: &crate::logs::write::raccoon::database::accountdb::account::ACCOUNT, offer: &str, play: serde_json::Value) {
        let queue_id = match play["data"]["play_queue_id"].as_str() {
            Some(id) => {
                println!("queued, id={} pos={:?}", id, play["data"]["queue_pos"]);
                id.to_string()
            }
            None => {
                let result = play["data"]["result"].as_str().expect("no queue id and no result");
                println!("instant slot, decrypting");
                let server_data = super::decrypt_result(result).expect("decrypt");
                let (_, bs_sc_id, gl_key, play_config, ws_url, ws_token) = parse_server_data(server_data).expect("parse");
                let rtcs = get_rtcs(&ws_url, &ws_token, &a.sn, &gl_key, bs_sc_id, play_config, &offer).await.expect("get_rtcs");
                println!("ANSWER: {}", rtcs.answer);
                println!("CANDIDATES: {:?}", rtcs.candidates);
                println!("E2E COMPLETE");
                return;
            }
        };

        let mut pos = 1;
        for i in 0..MAX_POLLS {
            pos = get_pos(&a.token, &a.sn, &queue_id).await.expect("get_pos");
            println!("queue pos {} (poll {})", pos, i + 1);
            if pos == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(POLL_SECS)).await;
        }
        assert_eq!(pos, 0, "queue never hit 0");

        println!("claiming");
        let claim = play_game(&a.token, &a.sn, GAME_KEY, Some(&queue_id)).await.expect("claim");
        let result = claim["data"]["result"].as_str().expect("claim had no result");
        let server_data = super::decrypt_result(result).expect("decrypt");
        let (_, bs_sc_id, gl_key, play_config, ws_url, ws_token) = parse_server_data(server_data).expect("parse");
        println!("claimed, gl_key={} ws={}", gl_key, ws_url);

        let rtcs = get_rtcs(&ws_url, &ws_token, &a.sn, &gl_key, bs_sc_id, play_config, &offer).await.expect("get_rtcs");
        println!("ANSWER: {}", rtcs.answer);
        println!("CANDIDATES: {:?}", rtcs.candidates);
        println!("E2E COMPLETE");
    }
}
