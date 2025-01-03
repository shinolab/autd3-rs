use std::io::{self, Write};

use autd3::{driver::link::Link, prelude::*};

use super::{
    audio_file::*, bessel::*, custom::*, fir::*, flag::*, focus::*, group::*, holo::*, plane::*,
    stm::*, user_defined_gain_modulation::*,
};

pub async fn run<L: Link>(mut autd: Controller<L>) -> anyhow::Result<()> {
    type Test<L> = (
        &'static str,
        fn(
            &'_ mut Controller<L>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<bool>> + '_>>,
    );

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version().await?.iter().for_each(|firm_info| {
        println!("{}", firm_info);
    });
    println!("============================================");

    let mut examples: Vec<Test<_>> = vec![
        ("Single focus test", |autd| Box::pin(focus(autd))),
        ("Bessel beam test", |autd| Box::pin(bessel(autd))),
        ("Plane wave test", |autd| Box::pin(plane(autd))),
        ("Wav modulation test", |autd| Box::pin(audio_file(autd))),
        ("FociSTM test", |autd| Box::pin(foci_stm(autd))),
        ("GainSTM test", |autd| Box::pin(gain_stm(autd))),
        ("Multiple foci test", |autd| Box::pin(holo(autd))),
        ("FIR test", |autd| Box::pin(fir(autd))),
        ("User-defined Gain & Modulation test", |autd| {
            Box::pin(user_defined(autd))
        }),
        ("Flag test", |autd| Box::pin(flag(autd))),
        ("Custom Gain test", |autd| Box::pin(custom(autd))),
        ("Group (by Transducer) test", |autd| {
            Box::pin(group_by_transducer(autd))
        }),
    ];
    if autd.num_devices() >= 2 {
        examples.push(("Group (by Device) test", |autd| {
            Box::pin(group_by_device(autd))
        }));
    }

    loop {
        examples.iter().enumerate().for_each(|(i, (name, _))| {
            println!("[{}]: {}", i, name);
        });
        println!("[Others]: Finish");
        color_print::cprint!("<green><bold>Choose number: ");
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        match s.trim().parse::<usize>() {
            Ok(i) if i < examples.len() => {
                if !(examples[i].1)(&mut autd).await? {
                    eprintln!("Failed to send data");
                }
            }
            _ => break,
        }

        println!("press any key to finish...");
        let mut _s = String::new();
        io::stdin().read_line(&mut _s)?;

        autd.send((Static::new(), Null::default())).await?;
        autd.send(Silencer::default()).await?;
    }

    autd.close().await?;

    Ok(())
}
