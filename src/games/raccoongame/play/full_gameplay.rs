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

const QUEUE_MAX_POLLS: u32 = 600;
const QUEUE_POLL_SECS: u64 = 3;

fn decrypt_result(result_b64: &str) -> anyhow::Result<Value> {
    let mut data = base64::engine::general_purpose::STANDARD.decode(result_b64)?;
    if data.len() == 0 || data.len() % 16 != 0 {
        anyhow::bail!("result blob is {} bytes, not a multiple of 16", data.len());
    }

    let cipher = Aes256::new(RESULT_KEY.into());
    for i in 0..data.len() / 16 {
        let block_start = i * 16;
        let prev_start = if i == 0 { 0 } else { block_start - 16 };
        let mut block: [u8; 16] = data[block_start..block_start + 16].try_into()?;
        let prev: [u8; 16] = if i == 0 {
            *RESULT_IV
        } else {
            data[prev_start..prev_start + 16].try_into()?
        };
        cipher.decrypt_block((&mut block).into());
        for (b, p) in block.iter_mut().zip(prev.iter()) {
            *b ^= p;
        }
        data[block_start..block_start + 16].copy_from_slice(&block);
    }

    let pad = *data.last().ok_or_else(|| anyhow::anyhow!("result blob empty"))? as usize;
    if pad == 0 || pad > 16 || pad > data.len() {
        anyhow::bail!("result blob has bogus pkcs7 padding byte {}", pad);
    }
    data.truncate(data.len() - pad);

    let parsed = serde_json::from_slice::<Value>(&data)?;
    if !parsed.is_object() {
        anyhow::bail!("decrypted result is not an object, raccoon sent soup");
    }
    Ok(parsed)
}


pub async fn get_rtc(
    token: &str,
    sn: &str,
    game_key: &str,
    offer_sdp: &str,
) -> anyhow::Result<Rtcs> {
    crate::vlog!("[get_rtc] inputs: sn={} game_key={}", sn, game_key);

    let _ = check_cost(token, sn, game_key).await?;

    let play = play_game(token, sn, game_key, None).await?;

    let mut server_data = if play["status"].as_u64() == Some(201)
        || (play["status"].as_u64() == Some(200) && play["data"]["play_queue_id"].is_string())
    {
        let queue_id = play["data"]["play_queue_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("got queued but no queue id came with it"))?
            .to_string();
        crate::vlog!("[get_rtc] queued under {}, polling", queue_id);

        let mut pos = 1;
        for i in 0..QUEUE_MAX_POLLS {
            pos = get_pos(token, sn, &queue_id).await?;
            crate::vlog!("[get_rtc] queue pos {} (poll {})", pos, i + 1);
            if pos == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(QUEUE_POLL_SECS)).await;
        }
        if pos != 0 {
            anyhow::bail!("queue never hit 0 after {} polls, giving up", QUEUE_MAX_POLLS);
        }

        let claim = play_game(token, sn, game_key, Some(&queue_id)).await?;
        let result = claim["data"]["result"].as_str().ok_or_else(|| {
            anyhow::anyhow!("queue claim had no result, claim status {}", claim["status"])
        })?;
        decrypt_result(result)?
    } else {
        let result = play["data"]["result"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("playGame had no result, status {}", play["status"]))?;
        decrypt_result(result)?
    };

    let sc_id = server_data["sc_id"]
        .as_str()
        .or_else(|| server_data["play_id"].as_str())
        .ok_or_else(|| anyhow::anyhow!("server_data missing sc_id, wtf raccoon"))?
        .to_string();
    let bs_sc_id = server_data["bs_sc_id"]
        .as_str()
        .unwrap_or(&sc_id)
        .to_string();
    let gl_key = server_data["gl_key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing gl_key, no game node assigned"))?
        .to_string();
    let play_config = server_data["play_config"].clone();
    let message_server = &server_data["message_server"];
    let ws_url = message_server["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing message_server.url, no signaling link"))?
        .to_string();
    let ws_token = message_server["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("server_data missing message_server.token"))?
        .to_string();

    crate::vlog!("[get_rtc] server ready sc_id={} gl_key={} ws={}", bs_sc_id, gl_key, ws_url);

    let rtcs = get_rtcs(&ws_url, &ws_token, sn, &gl_key, &bs_sc_id, play_config, offer_sdp).await?;

    crate::vlog!("[get_rtc] done, answer + {} candidates", rtcs.candidates.len());
    Ok(rtcs)
}
