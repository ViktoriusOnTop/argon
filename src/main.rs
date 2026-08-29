use crate::logs::make_folder::make_logging_folder;

pub mod games;
pub mod logs;

#[tokio::main]
async fn main() {
    let _ = make_logging_folder();
}