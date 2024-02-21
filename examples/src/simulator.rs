mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_simulator::Simulator;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .add_device(AUTD3::new(Vector3::new(AUTD3::DEVICE_WIDTH, 0.0, 0.0)))
        .open(Simulator::builder(8080))
        .await?;

    tests::run(autd).await
}
