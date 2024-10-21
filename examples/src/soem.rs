mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_soem::{Status, SOEM};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(SOEM::builder().with_err_handler(|slave, status| {
            eprintln!("slave[{}]: {}", slave, status);
            if status == Status::Lost {
                // You can also wait for the link to recover, without exitting the process
                std::process::exit(-1);
            }
        }))
        .await?;

    tests::run(autd).await
}
