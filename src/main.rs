use tokio::net::TcpListener;
use crate::logs::make_folder::make_logging_folder;
use std::process::Command;
use axum::Router;
use axum::routing::{get, post};
use axum_governor::{nz, GovernorConfigBuilder, GovernorLayer, PeerIp};
use clap::Parser;
use axum_governor::Quota;
use meilisearch_sdk::client::Client;
use tokio::sync::OnceCell;
use crate::api::argon::info::get_info;
use crate::api::raccoon::account::get_account;
use crate::api::raccoon::search::search;
use crate::games::raccoongame::searching::meilisearch::setup_woooo;
use crate::games::raccoongame::pool::handler::POOL;

pub mod games;
pub mod logs;
pub mod api;

#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => {
        if crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

#[derive(Parser)]
#[command(name = "argon")]
#[command(about = "Runs argon without docker")]
struct Args {
    #[arg(long)]
    no_docker: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    no_server: bool,
}

static MEILI_CLIENT: OnceCell<meilisearch_sdk::client::Client> = OnceCell::const_new();
static ACCOUNT_POOL: OnceCell<POOL> = OnceCell::const_new();
static VERBOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn get_pool() -> OnceCell<POOL> {
    ACCOUNT_POOL.clone()
}


#[tokio::main]
async fn main() {
    let _ = make_logging_folder();

    let args = Args::parse();

    let argon = r#"░█████╗░██████╗░░██████╗░░█████╗░███╗░░██╗
██╔══██╗██╔══██╗██╔════╝░██╔══██╗████╗░██║
███████║██████╔╝██║░░██╗░██║░░██║██╔██╗██║
██╔══██║██╔══██╗██║░░╚██╗██║░░██║██║╚████║
██║░░██║██║░░██║╚██████╔╝╚█████╔╝██║░╚███║
╚═╝░░╚═╝╚═╝░░╚═╝░╚═════╝░░╚════╝░╚═╝░░╚══╝
    "#;
    println!("welcome to");
    println!("{}", argon);
    println!("if something breaks or doesnt work then PLEASE PLEASE PLEASE make an issue https://github.com/ViktoriusOnTop/argon/issues or a PR if your a rustacean");

    if args.no_server {
        println!();
        println!();
        println!("are you mad wdym no server argon is a server");
        std::process::exit(0);
    }
    if args.verbose {
        VERBOSE.store(true, std::sync::atomic::Ordering::Relaxed);
        println!("verbose mode on E:");
    }

    let _ = ACCOUNT_POOL.get_or_init(|| async { POOL::new().await }).await;

    if !args.no_docker {
        if is_docker_cli_installed() {
            MEILI_CLIENT.get_or_init(|| async {
                setup_woooo().await.expect("Failed to initialize Meilisearch client")
            }).await;
            has_docker().await
        } else {
            eprintln!("Docker daemon is not installed. Please install docker or run argon in --no-docker mode. (sacrifices functionality, no search");
            std::process::exit(1);
        }
    }else {
        println!("Running argon without docker, and in turn without meilisearch. (sacrifices functionality, no search)");
        no_docker().await
    }
}

pub fn get_client() -> OnceCell<Client> {
    MEILI_CLIENT.clone()
}

async fn has_docker(){
    match setup_woooo().await {
        Ok(_) => setup_axum().await,
        Err(e) => eprintln!("milli failed :? {:?}", e),
    }
    setup_axum().await;
}

async fn no_docker(){
    setup_axum().await;
}

async fn setup_axum(){
    println!("setting up axum not millisearch jetbrains autocomplete ");
    let search_governor = GovernorConfigBuilder::default()
            .with_extractor(PeerIp::default())
            .expect_connect_info()
            .quota_default(
                Quota::requests_per_second(nz!(10u32),
                ))
            .finish()
            .unwrap();

    let account_governor = GovernorConfigBuilder::default()
            .with_extractor(PeerIp::default())
            .expect_connect_info()
            .quota_default(
                Quota::requests_per_hour(nz!(5u32),
                ))
            .finish()
            .unwrap();

    let information_governor = GovernorConfigBuilder::default()
            .with_extractor(PeerIp::default())
            .expect_connect_info()
            .quota_default(
                Quota::requests_per_second(nz!(60u32),
                ))
            .finish()
            .unwrap();

    let search_app: Router<()> = Router::new()
        .route("/api/raccoon/search", post(search))
        .layer(GovernorLayer::new(search_governor));

    let account_app: Router<()> = Router::new()
        .route("/api/raccoon/account", get(get_account))
        .layer(GovernorLayer::new(account_governor));

    let info_app: Router<()> = Router::new()
        .route("/api/argon/info", get(get_info))
        .layer(GovernorLayer::new(information_governor));

    let app = Router::new()
        .merge(search_app)
        .merge(account_app)
        .merge(info_app);

    let listener = TcpListener::bind("0.0.0.0:1818").await.unwrap(); //argon's atomic number is 18, clever ik

    println!("done or dumb");
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.expect("app start failed, is viktoriusontop stupid? :(");
}

fn is_docker_cli_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
