mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_soem::{Status, SOEM};

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(
            SOEM::builder().with_err_handler(|slave, status| match status {
                Status::Error(msg) => eprintln!("Error [{}]: {}", slave, msg),
                Status::Lost(msg) => {
                    eprintln!("Lost [{}]: {}", slave, msg);
                    // You can also wait for the link to recover, without exitting the process
                    std::process::exit(-1);
                }
                Status::StateChanged(msg) => eprintln!("StateChanged [{}]: {}", slave, msg),
            }),
        )
        .await?;

    tests::run(autd).await
}
