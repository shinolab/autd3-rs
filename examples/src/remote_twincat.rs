mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_twincat::RemoteTwinCAT;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(RemoteTwinCAT::builder("0.0.0.0.0.0"))
        .await?;

    tests::run(autd).await
}
