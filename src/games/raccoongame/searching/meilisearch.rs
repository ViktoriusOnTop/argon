use std::collections::HashMap;
use bollard::models::ContainerCreateBody;
use bollard::Docker;
use futures_util::{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{TryStreamExt}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}};
use std::time::Duration;
use bollard::config::{HostConfig, PortBinding};
use bollard::query_parameters::CreateContainerOptions;
use meilisearch_sdk::client::Client;
use serde::{Deserialize, Serialize};
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
    let docker = Docker::connect_with_local_defaults()?;

    docker
        .create_image(
            Some(
                bollard::query_parameters::CreateImageOptionsBuilder::default()
                    .from_image(IMAGE)
                    .build(),
            ),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await?;

    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        String::from("7700/tcp"),
        Some(vec![PortBinding {
            host_ip: Some(String::from("127.0.0.1")),
            host_port: Some(String::from("7700")),
        }]),
    );

    let meili = ContainerCreateBody {
        image: Some(String::from(IMAGE)),
        tty: Some(true),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        open_stdin: Some(true),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        }),
        ..Default::default()
    };

    let create_options = CreateContainerOptions {
        name: Some(CONTAINER_NAME.to_string()),
        platform: "".to_string(),
    };

    let create_result = docker
        .create_container(
            Some(create_options),
            meili,
        )
        .await;

    match create_result {
        Ok(_) => println!("it workdeth"),
        Err(bollard::errors::Error::DockerResponseServerError { status_code: 409, .. }) => {
            println!("you ran argon before :D");
        }
        Err(e) => return Err(e.into()),
    }

    let start_result = docker
        .start_container(
            CONTAINER_NAME,
            None::<bollard::query_parameters::StartContainerOptions>,
        )
        .await;

    match start_result {
        Ok(_) => println!("Meilisearch is running!"),
        Err(bollard::errors::Error::DockerResponseServerError { status_code: 304, .. }) => {
            println!("meili mommy is already running.");
        }
        Err(e) => return Err(e.into()),
    }

    println!("gonna take a nap rq");
    tokio::time::sleep(Duration::from_secs(30)).await; // give it time to start
    println!("alright im back, where were we");

    let ms_client = Client::new(MEILI_URL, Some("key"));
    let index = ms_client?.index("games");

    let outcome = index.add_documents(yoit(), Some("id")).await;

    match outcome {
        Ok(task) => println!("sendeth {:?}", task),
        Err(e) => eprintln!("!sendeth {}", e),
    }

    Ok(ms_client)
}

pub async fn search_meilisearch(client: &Client, query: &str) -> anyhow::Result<Vec<GAME>> {
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

    Ok(games)
}