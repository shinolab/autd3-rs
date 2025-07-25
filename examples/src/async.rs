use anyhow::Result;

use autd3::r#async::Controller;
use autd3::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }; 2],
        Nop::new(),
    )
    .await?;

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version().await?.iter().for_each(|firm_info| {
        println!("{firm_info}");
    });
    println!("============================================");

    autd.send((
        Sine {
            freq: 150. * Hz,
            option: Default::default(),
        },
        Focus {
            pos: Point3::new(90., 70., 150.),
            option: Default::default(),
        },
    ))
    .await?;

    println!("Press enter to exit...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.close().await?;

    Ok(())
}
