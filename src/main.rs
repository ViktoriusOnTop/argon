use crate::logs::make_folder::make_logging_folder;
use std::process::Command;
use clap::Parser;

pub mod games;
pub mod logs;

#[derive(Parser)]
#[command(name = "argon")]
#[command(about = "Runs argon without docker")]
struct Args {
    #[arg(long)]
    no_docker: bool,
}

#[tokio::main]
async fn main() {
    let _ = make_logging_folder();

    let args = Args::parse();

    if !args.no_docker {
        if is_docker_cli_installed() {

        } else {
            eprintln!("Docker daemon is not installed. Please install docker or run argon in --no-docker mode. (sacrifices functionality, no search");
            std::process::exit(1);
        }
    }else {
        println!("Running argon without docker, and in turn without meilisearch. (sacrifices functionality, no search)");

    }

}

fn is_docker_cli_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
