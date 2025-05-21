use autd3::prelude::*;

use autd3_protobuf::lightweight::Controller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut autd = Controller::open(
        [AUTD3::default(), AUTD3::default()],
        "127.0.0.1:8080".parse()?,
    )
    .await?;

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version().await?.iter().for_each(|firm_info| {
        println!("{}", firm_info);
    });
    println!("============================================");

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus {
        pos: center,
        option: Default::default(),
    };
    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g)).await?;

    {
        // GainSTM requires `autd3_protobuf::lightweight::IntoLightweightGain::into_lightweight()`
        use autd3_protobuf::lightweight::IntoLightweightGain;
        let stm = GainSTM {
            gains: vec![
                Null {}.into_lightweight(),
                Focus {
                    pos: center,
                    option: Default::default(),
                }
                .into_lightweight(),
            ],
            config: 1.0 * Hz,
            option: Default::default(),
        };
        autd.send(stm).await?;
    }

    println!("Press enter to exit...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.close().await?;

    Ok(())
}
