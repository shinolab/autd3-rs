mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_soem::{Status, SOEM};

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(
            SOEM::builder().with_err_handler(|slave, status| match status {
                Status::Lost => {
                    eprintln!("slave[{}]: {}", slave, status);
                    // You can also wait for the link to recover, without exitting the process
                    std::process::exit(-1);
                }
                _ => eprintln!("slave[{}]: {}", slave, status),
            }),
        )
        .await?;

    tests::run(autd).await
}
