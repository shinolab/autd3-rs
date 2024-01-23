mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_soem::SOEM;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(SOEM::builder().with_on_lost(|msg| {
            eprintln!("Unrecoverable error occurred: {msg}");
            std::process::exit(-1);
        }))
        .await?;

    tests::run(autd).await
}
