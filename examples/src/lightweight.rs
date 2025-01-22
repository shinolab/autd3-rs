use autd3::prelude::*;

use autd3_protobuf::lightweight::LightweightClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = LightweightClient::builder([AUTD3::default()])
        .open("127.0.0.1:8080".parse()?)
        .await?;

    println!("======== AUTD3 firmware information ========");
    client
        .firmware_version()
        .await?
        .iter()
        .for_each(|firm_info| {
            println!("{}", firm_info);
        });
    println!("============================================");

    client
        .send(Sine {
            freq: 150. * Hz,
            option: Default::default(),
        })
        .await?;
    client.send(Focus::new(Point3::new(90., 70., 150.))).await?;

    println!("Press enter to exit...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    client.close().await?;

    Ok(())
}
