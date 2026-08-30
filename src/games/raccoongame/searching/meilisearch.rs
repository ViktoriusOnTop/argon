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

    crate::dlog!("inspecting for an index");
    let inspect_output = Command::new("docker")
        .args(["image", "inspect", "getmeili/meilisearch:latest"])
        .output()
        .expect("docker inspect fumbled, make an issue");
    crate::dlog!("[meili] image inspect inputs: args={:?} stdout={} stderr={}",
        ["image", "inspect", "getmeili/meilisearch:latest"],
        String::from_utf8_lossy(&inspect_output.stdout).trim_end(),
        String::from_utf8_lossy(&inspect_output.stderr).trim_end());

    if inspect_output.status.success() {
        crate::dlog!("found it");
        crate::dlog!("[meili] image already exists, skippin the pull");
    } else {
        crate::dlog!("huh. musta got away");
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

    println!("im not going to stab meili with a javelin");
    let mut healthy = false;
    for attempt in 0..30 {
        match reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?
            .get("http://localhost:7700/health")
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                crate::dlog!("im back i totally didn't poke meili {} times for it to wake up", attempt + 1);
                healthy = true;
                break;
            }
            Ok(res) => crate::dlog!("meili  said yo mama is fat, and also {} on my not a poke number {}", res.status(), attempt + 1),
            Err(_) => crate::dlog!("/health not answerin on poke with my javelin that i didnt do, {}, not ready yet", attempt + 1),
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    if !healthy {
        anyhow::bail!("meili never woke up... it just died. make an issue");
    }

    let ms_client = Client::new(MEILI_URL, Some("key"))?;

    let stats = ms_client.index("games").get_stats().await;
    let doc_count = match stats {
        Ok(s) => s.number_of_documents,
        Err(_) => 0,
    };

    if doc_count > 0 {
        crate::dlog!("[meili] index games already has {} docs, skippin re-indexin", doc_count);
    } else {
        let index = ms_client.index("games");

        let docs = extract().await?;
        crate::dlog!("[meili] add_documents inputs: url={} index=\"games\" primary_key=\"id\" doc_count={} docs={:?}",
            MEILI_URL, docs.len(), docs);

        let outcome = index.add_documents(&docs, Some("id")).await;

        match outcome {
            Ok(task) => println!("sendeth {:?}", task),
            Err(e) => eprintln!("!sendeth {}", e),
        }
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