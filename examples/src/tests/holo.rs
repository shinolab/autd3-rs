use autd3::{driver::link::Link, prelude::*};
use autd3_gain_holo::*;

use colored::*;
use std::io::{self, Write};

pub async fn holo(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
    let p = Vector3::new(30. * mm, 0., 0.);
    let backend = NalgebraBackend::<Sphere>::new()?;
    let target_amp = 2.5e3 * autd.geometry().num_devices() as f32 * Pa;
    let foci = [(center + p, target_amp), (center - p, target_amp)];

    let mut gains: Vec<(&str, Box<dyn autd3::driver::datagram::Gain>)> = vec![
        ("GS", Box::new(GS::new(backend.clone(), foci))),
        ("GSPAT", Box::new(GSPAT::new(backend.clone(), foci))),
        ("Naive", Box::new(Naive::new(backend.clone(), foci))),
        ("LM", Box::new(LM::new(backend.clone(), foci))),
        ("Greedy", Box::new(Greedy::<Sphere>::new(foci))),
    ];

    gains.iter().enumerate().for_each(|(i, (name, _))| {
        println!("[{}]: {}", i, name);
    });
    println!("[Others]: GSPAT");
    print!("{}", "Choose number: ".green().bold());
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let idx = match s.trim().parse::<usize>() {
        Ok(i) if i < gains.len() => i,
        _ => 1,
    };

    let m = Sine::new(150. * Hz);
    let g = gains.swap_remove(idx).1;
    autd.send((m, g)).await?;

    Ok(true)
}
