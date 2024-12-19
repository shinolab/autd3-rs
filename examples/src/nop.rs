mod tests;

use anyhow::Result;

use autd3::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let autd = Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
        .open(Nop::builder())
        .await?;

    tests::run(autd).await
}
