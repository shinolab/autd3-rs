use autd3::{driver::link::Link, prelude::*};
use autd3_gain_holo::*;

use colored::*;
use std::io::{self, Write};

pub async fn holo(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let m = Sine::new(150. * Hz);

    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
    let p = Vector3::new(30. * mm, 0., 0.);

    println!("[0]: GS");
    println!("[1]: GSPAT");
    println!("[2]: LSS");
    println!("[3]: LM");
    println!("[4]: Greedy");
    println!("[Others]: GS-PAT");
    print!("{}", "Choose number: ".green().bold());
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;

    let backend = NalgebraBackend::<Sphere>::new().unwrap();

    let target_amp = 2.5e3 * autd.geometry().num_devices() as f32 * Pa;
    match s.trim().parse::<usize>() {
        Ok(0) => {
            let g = GS::new(
                backend,
                [(center + p, target_amp), (center - p, target_amp)],
            );
            autd.send((m, g)).await?
        }
        Ok(1) => {
            let g = GSPAT::new(
                backend,
                [(center + p, target_amp), (center - p, target_amp)],
            );
            autd.send((m, g)).await?
        }
        Ok(2) => {
            let g = LSS::new(
                backend,
                [(center + p, target_amp), (center - p, target_amp)],
            );
            autd.send((m, g)).await?
        }
        Ok(3) => {
            let g = LM::new(
                backend,
                [(center + p, target_amp), (center - p, target_amp)],
            );
            autd.send((m, g)).await?
        }
        Ok(4) => {
            let g = Greedy::<Sphere>::new([(center + p, target_amp), (center - p, target_amp)]);
            autd.send((m, g)).await?
        }
        _ => {
            let g = GSPAT::new(
                backend,
                [(center + p, target_amp), (center - p, target_amp)],
            );
            autd.send((m, g)).await?
        }
    };

    Ok(true)
}
