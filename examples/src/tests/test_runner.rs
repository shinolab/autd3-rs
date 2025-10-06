use std::io::{self, Write};

use autd3::{core::link::Link, prelude::*};

use super::{
    audio_file::*, bessel::*, custom::*, fir::*, flag::*, focus::*, group::*, holo::*, plane::*,
    stm::*, user_defined_gain_modulation::*,
};

pub fn run<L: Link>(mut autd: Controller<L>) -> Result<(), Box<dyn std::error::Error>> {
    type Test<L> = (
        &'static str,
        fn(&'_ mut Controller<L>) -> Result<(), Box<dyn std::error::Error>>,
    );

    println!("======== AUTD3 firmware information ========");
    autd.firmware_version()?.iter().for_each(|firm_info| {
        println!("{firm_info}");
    });
    println!("============================================");

    let mut examples: Vec<Test<_>> = vec![
        ("Single focus test", |autd| focus(autd)),
        ("Bessel beam test", |autd| bessel(autd)),
        ("Plane wave test", |autd| plane(autd)),
        ("Wav modulation test", |autd| audio_file(autd)),
        ("FociSTM test", |autd| foci_stm(autd)),
        ("GainSTM test", |autd| gain_stm(autd)),
        ("Multiple foci test", |autd| holo(autd)),
        ("FIR test", |autd| fir(autd)),
        ("User-defined Gain & Modulation test", |autd| {
            user_defined(autd)
        }),
        ("Flag test", |autd| flag(autd)),
        ("Custom Gain test", |autd| custom(autd)),
        ("Group (by Transducer) test", |autd| {
            group_by_transducer(autd)
        }),
    ];
    if autd.num_devices() >= 2 {
        examples.push(("Group (by Device) test", |autd| group_by_device(autd)));
    }

    loop {
        examples.iter().enumerate().for_each(|(i, (name, _))| {
            println!("[{i}]: {name}");
        });
        println!("[Others]: Finish");
        print!("Choose number: ");
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        match s.trim().parse::<usize>() {
            Ok(i) if i < examples.len() => {
                (examples[i].1)(&mut autd)?;
            }
            _ => break,
        }

        println!("press any key to finish...");
        let mut _s = String::new();
        io::stdin().read_line(&mut _s)?;

        autd.send((Static::default(), Null))?;
        autd.send(Silencer::default())?;
    }

    autd.close()?;

    Ok(())
}
