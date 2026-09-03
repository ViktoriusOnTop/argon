use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use heed::types::{SerdeJson, Str};
use heed::{Database, Env, EnvOpenOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::logs::make_folder::get_platform_log_dir;
use crate::games::raccoongame::play::probe::GameProbe;
use crate::logs::write::raccoon::database::accountdb::account::{PublicAccount, ACCOUNT};

static ACCOUNT_DB: OnceLock<AccountDb> = OnceLock::new();

const STALE_SECS: u64 = 3600;

#[derive(Serialize, Deserialize, Clone)]
pub struct ActiveAccount {
    pub public_id: String,
    pub sn: String,
    pub token: String,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaySession {
    pub game_key: String,
    pub offer_sdp: String,
    pub queue_id: String,
    pub created_at: u64,
}

pub struct AccountDb {
    env: Env,
    accounts: Database<Str, SerdeJson<ACCOUNT>>,
    games: Database<Str, SerdeJson<GameProbe>>,
    active: RwLock<HashMap<String, ActiveAccount>>,
    sessions: RwLock<HashMap<String, PlaySession>>,
}

impl AccountDb {
    fn open() -> anyhow::Result<Self> {
        let dir = db_path()?;
        fs::create_dir_all(&dir)?;
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(1024 * 1024 * 1024)
                .max_dbs(3)
                .open(dir)?
        };
        let mut wtxn = env.write_txn()?;
        let accounts = env.create_database(&mut wtxn, Some("accounts"))?;
        let games = env.create_database(&mut wtxn, Some("games"))?;
        wtxn.commit()?;
        Ok(Self {
            env,
            accounts,
            games,
            active: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub fn set_games(&self, games: &[GameProbe]) -> anyhow::Result<()> {
        let mut wtxn = self.env.write_txn()?;
        for game in games {
            self.games.put(&mut wtxn, &game.game_key, game)?;
        }
        wtxn.commit()?;
        Ok(())
    }

    pub fn clear_games(&self) -> anyhow::Result<()> {
        let mut wtxn = self.env.write_txn()?;
        self.games.clear(&mut wtxn)?;
        wtxn.commit()?;
        Ok(())
    }

    pub fn is_playable(&self, game_key: &str) -> anyhow::Result<bool> {
        let rtxn = self.env.read_txn()?;
        Ok(self.games.get(&rtxn, game_key)?.map(|g| g.playable).unwrap_or(false))
    }

    pub fn playable_games(&self) -> anyhow::Result<usize> {
        let rtxn = self.env.read_txn()?;
        let mut count = 0;
        for row in self.games.iter(&rtxn)? {
            let (_, game) = row?;
            if game.playable {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn games_count(&self) -> anyhow::Result<usize> {
        let rtxn = self.env.read_txn()?;
        Ok(self.games.len(&rtxn)? as usize)
    }

    pub fn push_account(&self, account: &ACCOUNT) -> anyhow::Result<()> {
        let mut wtxn = self.env.write_txn()?;
        self.accounts.put(&mut wtxn, &account.public_id, account)?;
        wtxn.commit()?;
        crate::vlog!("[accountdb] push_account public_id={} stock={}", account.public_id, self.stock_len()?);
        Ok(())
    }

    pub fn pop_account(&self, public_id: &str) -> anyhow::Result<Option<PublicAccount>> {
        let mut wtxn = self.env.write_txn()?;
        let Some(account) = self.accounts.get(&wtxn, public_id)? else {
            wtxn.commit()?;
            return Ok(None);
        };
        let handed = ActiveAccount {
            public_id: public_id.to_string(),
            sn: account.sn.clone(),
            token: account.token.clone(),
            created_at: now_ms(),
        };
        let public = PublicAccount::from(&account);
        self.accounts.delete(&mut wtxn, public_id)?;
        wtxn.commit()?;
        self.active.write().unwrap().insert(public_id.to_string(), handed);
        crate::vlog!("[accountdb] pop_account public_id={} handed out, sn and token stuck in ram", public_id);
        Ok(Some(public))
    }

    pub fn any_public_id(&self) -> anyhow::Result<Option<String>> {
        let rtxn = self.env.read_txn()?;
        if let Some(row) = self.accounts.iter(&rtxn)?.next() {
            let (key, _) = row?;
            return Ok(Some(key.to_string()));
        }
        Ok(None)
    }

    pub fn active(&self, public_id: &str) -> anyhow::Result<Option<ActiveAccount>> {
        Ok(self.active.read().unwrap().get(public_id).cloned())
    }

    pub fn consume(&self, public_id: &str) -> anyhow::Result<bool> {
        let deleted = self.active.write().unwrap().remove(public_id).is_some();
        if deleted {
            crate::vlog!("[accountdb] consume public_id={} played and deleted, rip bozo", public_id);
        }
        Ok(deleted)
    }

    pub fn stock_len(&self) -> anyhow::Result<usize> {
        let rtxn = self.env.read_txn()?;
        Ok(self.accounts.len(&rtxn)? as usize)
    }

    pub fn all_accounts(&self) -> anyhow::Result<Vec<ACCOUNT>> {
        let rtxn = self.env.read_txn()?;
        let mut all = Vec::new();
        for row in self.accounts.iter(&rtxn)? {
            let (_, account) = row?;
            all.push(account);
        }
        Ok(all)
    }

    pub fn put_session(&self, public_id: &str, session: &PlaySession) -> anyhow::Result<()> {
        let mut session = session.clone();
        session.created_at = now_ms();
        crate::vlog!("[accountdb] put_session public_id={} queue_id={}", public_id, session.queue_id);
        self.sessions.write().unwrap().insert(public_id.to_string(), session);
        Ok(())
    }

    pub fn get_session(&self, public_id: &str) -> anyhow::Result<Option<PlaySession>> {
        Ok(self.sessions.read().unwrap().get(public_id).cloned())
    }

    pub fn delete_session(&self, public_id: &str) -> anyhow::Result<()> {
        self.sessions.write().unwrap().remove(public_id);
        crate::vlog!("[accountdb] delete_session public_id={}", public_id);
        Ok(())
    }

    pub fn sweep_stale(&self) -> usize {
        let now = now_ms();
        let mut active = self.active.write().unwrap();
        let a_before = active.len();
        active.retain(|_, a| now.saturating_sub(a.created_at) < STALE_SECS * 1000);
        let a_removed = a_before - active.len();
        drop(active);

        let mut sessions = self.sessions.write().unwrap();
        let s_before = sessions.len();
        sessions.retain(|_, s| now.saturating_sub(s.created_at) < STALE_SECS * 1000);
        let s_removed = s_before - sessions.len();

        if a_removed + s_removed > 0 {
            crate::vlog!("[accountdb] swept {} active + {} sessions", a_removed, s_removed);
        }
        a_removed + s_removed
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn db_path() -> anyhow::Result<PathBuf> {
    Ok(get_platform_log_dir()?.join("raccoon").join("accounts").join("database"))
}

pub fn account_db() -> anyhow::Result<&'static AccountDb> {
    if let Some(db) = ACCOUNT_DB.get() {
        return Ok(db);
    }
    let db = AccountDb::open()?;
    Ok(ACCOUNT_DB.get_or_init(|| db))
}

pub fn new_public_id(sn: &str, token: &str) -> String {
    let digest = Sha256::digest([sn.as_bytes(), token.as_bytes()].concat());
    let mut out = String::new();
    for b in digest {
        out.push_str(&format!("{:02x}", b));
    }
    crate::vlog!("[accountdb] new_public_id public_id={}", out);
    out
}
