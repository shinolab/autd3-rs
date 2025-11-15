use autd3::{core::link::Link, prelude::*};
use autd3_gain_holo::*;

use std::io::{self, Write};

pub fn holo(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::default())?;

    let target_amp = 2.5e3 * autd.num_devices() as f32 * Pa;
    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);
    let p = Vector3::new(30. * mm, 0., 0.);
    let foci = [(center + p, target_amp), (center - p, target_amp)];

    let mut gains: Vec<(&str, BoxedGain)> = vec![
        ("GS", BoxedGain::new(GS::new(foci, GSOption::default()))),
        (
            "GSPAT",
            BoxedGain::new(GSPAT::new(foci, GSPATOption::default())),
        ),
        (
            "Naive",
            BoxedGain::new(Naive::new(foci, NaiveOption::default())),
        ),
        (
            "Greedy",
            BoxedGain::new(Greedy::new(foci, GreedyOption::default())),
        ),
    ];

    gains.iter().enumerate().for_each(|(i, (name, _))| {
        println!("[{i}]: {name}");
    });
    println!("[Others]: GSPAT");
    print!("Choose number: ");
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let idx = s
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|&i| i < gains.len())
        .unwrap_or(1);

    let g = gains.swap_remove(idx).1;

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(())
}
