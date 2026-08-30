use std::{env, io};
use std::io::Write;
use std::process::Command;
use std::time::Duration;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
use crate::games::raccoongame::searching::extract_titles::extract;
use crate::games::raccoongame::searching::pull_games::yoit;

const IMAGE: &str = "getmeili/meilisearch:latest";
const CONTAINER_NAME: &str = "meili-mommy";
const MEILI_URL: &str = "http://localhost:7700";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GAME {
    pub id: u64,
    pub name: String,
    pub main_img: String,
    pub game_recommend_img: String,
    pub game_icon: String,
}

pub async fn setup_woooo() -> anyhow::Result<Client> {

    let docker_output = Command::new("docker")
        .args(["pull", "getmeili/meilisearch:latest"])
        .output()
        .expect("failed to execute pull meilisearch make an issue");
    println!("{}", docker_output.status);
    crate::dlog!("[meili] docker pull inputs: args={:?} stdout={} stderr={}",
        ["pull", "getmeili/meilisearch:latest"],
        String::from_utf8_lossy(&docker_output.stdout).trim_end(),
        String::from_utf8_lossy(&docker_output.stderr).trim_end());

    if !docker_output.status.success() {
        anyhow::bail!("docker pull failed, if docker you have docker then make an issue, or if you dont have it then make an issue, this shouldn't run without docker");
    }

    let mut current_dir = env::current_dir()
        .expect("Failed to get current directory")
        .to_string_lossy()
        .into_owned();

    if cfg!(target_os = "windows") {
        current_dir = current_dir.replace("\\", "/");
    }

    let _ = Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output();

    let start_status = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--name", CONTAINER_NAME,
            "-p", "7700:7700",
            "-e", "MEILI_MASTER_KEY=key",
            "-v", &format!("{}/meili_data:/meili_data", current_dir),
            "getmeili/meilisearch:latest",
        ])
        .output()
        .expect("docker didnt pulleth");

    println!("{}", start_status.status);
    crate::dlog!("[meili] docker run inputs: current_dir={} args={:?} stdout={} stderr={}",
        current_dir,
        ["run", "-d", "--rm", "--name", CONTAINER_NAME, "-p", "7700:7700", "-e", "MEILI_MASTER_KEY=key",
         "-v", &format!("{}/meili_data:/meili_data", current_dir), "getmeili/meilisearch:latest"],
        String::from_utf8_lossy(&start_status.stdout).trim_end(),
        String::from_utf8_lossy(&start_status.stderr).trim_end());

    if !start_status.status.success() {
        let err_msg = String::from_utf8_lossy(&start_status.stderr);
        anyhow::bail!("Failed to start Meilisearch container: {}", err_msg);
    }

    println!("gonna take a nap rq");
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("alright im back, where were we");

    let ms_client = Client::new(MEILI_URL, Some("key"))?;
    let index = ms_client.index("games");

    let docs = extract().await?;
    crate::dlog!("[meili] add_documents inputs: url={} index=\"games\" primary_key=\"id\" doc_count={} docs={:?}",
        MEILI_URL, docs.len(), docs);

    let outcome = index.add_documents(&docs, Some("id")).await;

    match outcome {
        Ok(task) => println!("sendeth {:?}", task),
        Err(e) => eprintln!("!sendeth {}", e),
    }

    Ok(ms_client)
}

pub async fn search_meilisearch(client: &Client, query: &str) -> anyhow::Result<Vec<GAME>> {
    crate::dlog!("[meili] search inputs: query={:?}", query);
    let index =client.index("games");

    let results = index
        .search()
        .with_query(query)
        .execute::<GAME>()
        .await?;

    let games: Vec<GAME> = results
        .hits
        .into_iter()
        .map(|hit| hit.result)
        .collect();

    crate::dlog!("[meili] search outputs: hit_count={} games={:?}", games.len(), games);

    Ok(games)
}