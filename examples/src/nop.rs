mod tests;

use anyhow::Result;

use autd3::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let autd = Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
        .open(Nop::builder())
        .await?;

    tests::run(autd).await
}
