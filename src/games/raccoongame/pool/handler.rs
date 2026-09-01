use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{Mutex, Notify};

use crate::games::raccoongame::pool::account::ACCOUNT;
use crate::games::raccoongame::raccoon_account::full_account_builder::build;
use crate::logs::read::raccoon::pool::get_pool_config::get_target_stock;
use crate::logs::write::raccoon::database::accountdb::account::ACCOUNT as DB_ACCOUNT;
use crate::logs::write::raccoon::database::accountdb::db as accountdb;

const WAIT_TIMEOUT: u64 = 30;
const MAX_BACKOFF_SECS: u64 = 60;
const MAX_WORKERS: u8 = 40;
const IDLE_CYCLES_BEFORE_EXIT: u32 = 60;

#[derive(Clone)]
pub struct POOL {
    state: Arc<Shared>,
}

struct Shared {
    accounts: Mutex<VecDeque<ACCOUNT>>,
    notify: Notify,
    worker_count: AtomicU8,
    inflight: AtomicUsize,
    target_stock: usize,
}

impl POOL {
    pub async fn new() -> POOL {
        let state = Arc::new(Shared {
            accounts: Mutex::new(load_stock()),
            notify: Notify::new(),
            worker_count: AtomicU8::new(0),
            inflight: AtomicUsize::new(0),
            target_stock: get_target_stock(),
        });
        let pool = POOL { state };
        pool.release_worker().await;
        pool
    }

    pub async fn get_account(&self) -> Option<ACCOUNT> {
        loop {
            let mut notified = Box::pin(self.state.notify.notified());
            notified.as_mut().enable();

            if let Some(account) = self.state.accounts.lock().await.pop_front() {
                return Some(account);
            }

            self.release_worker().await;
            match tokio::time::timeout(Duration::from_secs(WAIT_TIMEOUT), notified).await {
                Ok(_) => continue,
                Err(_) => return None,
            }
        }
    }

    pub async fn release_worker(&self) {
        let count = self.state.worker_count.fetch_add(1, Ordering::Relaxed);

        if count < 3 {
            println!("dispatching worker");
        } else if count < 10 {
            println!("you have over 3 workers this is really not necessary");
        } else if count < MAX_WORKERS {
            println!("THATS ENOUUGHHH");
        } else {
            println!("i hate you");
            self.state.worker_count.fetch_sub(1, Ordering::Relaxed);
            return;
        }

        let state = Arc::clone(&self.state);
        tokio::spawn(async move {
            worker_loop(state).await;
        });
    }

    pub async fn stock_len(&self) -> usize {
        self.state.accounts.lock().await.len()
    }
}

async fn worker_loop(state: Arc<Shared>) {
    let mut consecutive_fails: u32 = 0;
    let mut idle_cycles: u32 = 0;

    loop {
        let need = {
            let stock = state.accounts.lock().await;
            state.target_stock
                .saturating_sub(stock.len())
                .saturating_sub(state.inflight.load(Ordering::Relaxed))
        };

        if need == 0 {
            idle_cycles += 1;
            if idle_cycles >= IDLE_CYCLES_BEFORE_EXIT {
                state.worker_count.fetch_sub(1, Ordering::Relaxed);
                println!("worker goin home, pool is full");
                return;
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }
        idle_cycles = 0;

        state.inflight.fetch_add(1, Ordering::Relaxed);
        let result = build().await;
        state.inflight.fetch_sub(1, Ordering::Relaxed);

        match result {
            Ok((token, sn, email, password)) => {
                consecutive_fails = 0;
                let public_id = accountdb::new_public_id();
                let account = ACCOUNT::new(token.clone(), sn.clone(), email.clone(), password.clone(), public_id.clone());
                let record = DB_ACCOUNT { sn, token, email, password, public_id };
                match accountdb::account_db() {
                    Ok(db) => {
                        if let Err(e) = db.push_account(&record) {
                            eprintln!("couldnt push account to heed, account is lost to the void: {}", e);
                        }
                    }
                    Err(e) => eprintln!("couldnt open accountdb, account is lost to the void: {}", e),
                }
                let mut stock = state.accounts.lock().await;
                stock.push_back(account);
                drop(stock);
                state.notify.notify_waiters();
            }
            Err(_) => {
                consecutive_fails += 1;
                let backoff =
                    3u64.saturating_mul(1 << consecutive_fails.min(5)).min(MAX_BACKOFF_SECS);
                eprintln!("worker build failed x{}, nappin {}s", consecutive_fails, backoff);
                if crate::games::raccoongame::limiter::RETRY_BACKOFF_ENABLED {
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                }
            }
        }
    }
}

pub fn load_stock() -> VecDeque<ACCOUNT> {
    let mut stock = VecDeque::new();
    match accountdb::account_db() {
        Ok(db) => match db.all_accounts() {
            Ok(accounts) => {
                for a in accounts {
                    stock.push_back(ACCOUNT::new(a.token.clone(), a.sn.clone(), a.email.clone(), a.password.clone(), a.public_id.clone()));
                }
                println!("loaded {} accounts from heed", stock.len());
            }
            Err(e) => println!("couldnt read heed, starting with an empty pool: {}", e),
        },
        Err(e) => println!("couldnt open heed, starting with an empty pool: {}", e),
    }
    stock
}

