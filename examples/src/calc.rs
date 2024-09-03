mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_calc::Calc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let autd = Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    tests::run(autd).await
}
