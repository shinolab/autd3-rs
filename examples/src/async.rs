use anyhow::Result;

use autd3::prelude::*;
use autd3::r#async::Controller;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut autd =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
            .open(Nop::builder())
            .await?;

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version().await?.iter().for_each(|firm_info| {
        println!("{}", firm_info);
    });
    println!("============================================");

    autd.send((
        Sine::new(150. * Hz),
        Focus::new(Point3::new(90., 70., 150.)),
    ))
    .await?;

    println!("Press enter to exit...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.close().await?;

    Ok(())
}
