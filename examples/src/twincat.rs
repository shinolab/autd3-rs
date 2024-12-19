mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_twincat::TwinCAT;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder([AUTD3::new(Point3::origin())])
        .open(TwinCAT::builder())
        .await?;

    tests::run(autd).await
}
