use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use heed::types::{SerdeJson, Str};
use heed::{Database, Env, EnvOpenOptions};
use rand::distr::Alphanumeric;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::logs::make_folder::get_platform_log_dir;
use crate::logs::write::raccoon::database::accountdb::account::{PublicAccount, ACCOUNT};

static ACCOUNT_DB: OnceLock<AccountDb> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct ActiveAccount {
    pub public_id: String,
    pub sn: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaySession {
    pub game_key: String,
    pub offer_sdp: String,
    pub queue_id: String,
}

pub struct AccountDb {
    env: Env,
    accounts: Database<Str, SerdeJson<ACCOUNT>>,
    active: Database<Str, SerdeJson<ActiveAccount>>,
    sessions: Database<Str, SerdeJson<PlaySession>>,
}

impl AccountDb {
    fn open() -> anyhow::Result<Self> {
        let dir = db_path()?;
        fs::create_dir_all(&dir)?;
        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(1024 * 1024 * 1024)
                .max_dbs(4)
                .open(dir)?
        };
        let mut wtxn = env.write_txn()?;
        let accounts = env.create_database(&mut wtxn, Some("accounts"))?;
        let active = env.create_database(&mut wtxn, Some("active"))?;
        let sessions = env.create_database(&mut wtxn, Some("sessions"))?;
        wtxn.commit()?;
        Ok(Self { env, accounts, active, sessions })
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
        };
        let public = PublicAccount::from(&account);
        self.accounts.delete(&mut wtxn, public_id)?;
        self.active.put(&mut wtxn, public_id, &handed)?;
        wtxn.commit()?;
        crate::vlog!("[accountdb] pop_account public_id={} handed out, sn and token locked in the active db", public_id);
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
        let rtxn = self.env.read_txn()?;
        Ok(self.active.get(&rtxn, public_id)?)
    }

    pub fn consume(&self, public_id: &str) -> anyhow::Result<bool> {
        let mut wtxn = self.env.write_txn()?;
        let deleted = self.active.delete(&mut wtxn, public_id)?;
        wtxn.commit()?;
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
        let mut wtxn = self.env.write_txn()?;
        self.sessions.put(&mut wtxn, public_id, session)?;
        wtxn.commit()?;
        crate::vlog!("[accountdb] put_session public_id={} queue_id={}", public_id, session.queue_id);
        Ok(())
    }

    pub fn get_session(&self, public_id: &str) -> anyhow::Result<Option<PlaySession>> {
        let rtxn = self.env.read_txn()?;
        Ok(self.sessions.get(&rtxn, public_id)?)
    }

    pub fn delete_session(&self, public_id: &str) -> anyhow::Result<()> {
        let mut wtxn = self.env.write_txn()?;
        self.sessions.delete(&mut wtxn, public_id)?;
        wtxn.commit()?;
        crate::vlog!("[accountdb] delete_session public_id={}", public_id);
        Ok(())
    }
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

pub fn new_public_id() -> String {
    rand::rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect()
}
