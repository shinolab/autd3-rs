mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_soem::RemoteSOEM;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(RemoteSOEM::builder("127.0.0.1:8080".parse()?))
        .await?;

    tests::run(autd).await
}
