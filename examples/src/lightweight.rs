use autd3::prelude::*;

use autd3_protobuf::lightweight::Controller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut autd = Controller::open([AUTD3::default()], "127.0.0.1:8080".parse()?).await?;

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version().await?.iter().for_each(|firm_info| {
        println!("{}", firm_info);
    });
    println!("============================================");

    autd.send(Sine {
        freq: 150. * Hz,
        option: Default::default(),
    })
    .await?;
    autd.send(Focus {
        pos: autd.center() + Vector3::new(0., 0., 150.),
        option: Default::default(),
    })
    .await?;

    println!("Press enter to exit...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.close().await?;

    Ok(())
}
