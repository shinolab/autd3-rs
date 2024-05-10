use autd3::prelude::*;
use autd3_gain_holo::*;

use colored::*;
use std::{
    io::{self, Write},
    sync::Arc,
};

pub async fn holo(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let m = Sine::new(150. * Hz);

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
    let p = Vector3::new(30. * MILLIMETER, 0., 0.);

    println!("[0]: SDP");
    println!("[1]: GS");
    println!("[2]: GSPAT");
    println!("[3]: LSS");
    println!("[4]: LM");
    println!("[5]: Greedy");
    println!("[Others]: GS-PAT");
    print!("{}", "Choose number: ".green().bold());
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;

    let backend = Arc::new(NalgebraBackend::default());

    let target_amp = 2.5e3 * autd.geometry.num_devices() as f64 * Pascal;
    match s.trim().parse::<usize>() {
        Ok(0) => {
            let g = SDP::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        Ok(1) => {
            let g = GS::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        Ok(2) => {
            let g = GSPAT::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        Ok(3) => {
            let g = LSS::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        Ok(4) => {
            let g = LM::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        Ok(5) => {
            let g = Greedy::default()
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
        _ => {
            let g = GSPAT::new(backend)
                .add_focus(center + p, target_amp)
                .add_focus(center - p, target_amp);
            autd.send((m, g)).await?
        }
    };

    Ok(true)
}
