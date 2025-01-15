use autd3::{
    core::link::Link,
    driver::datagram::{BoxedGain, IntoBoxedGain},
    prelude::*,
};
use autd3_gain_holo::*;

use std::io::{self, Write};

pub fn holo(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);
    let p = Vector3::new(30. * mm, 0., 0.);
    let backend = std::sync::Arc::new(NalgebraBackend::default());
    let target_amp = 2.5e3 * autd.num_devices() as f32 * Pa;
    let foci = [(center + p, target_amp), (center - p, target_amp)];

    let mut gains: Vec<(&str, BoxedGain)> = vec![
        ("GS", GS::new(backend.clone(), foci).into_boxed()),
        ("GSPAT", GSPAT::new(backend.clone(), foci).into_boxed()),
        ("Naive", Naive::new(backend.clone(), foci).into_boxed()),
        ("LM", LM::new(backend.clone(), foci).into_boxed()),
        ("Greedy", Greedy::<Sphere>::new(foci).into_boxed()),
    ];

    gains.iter().enumerate().for_each(|(i, (name, _))| {
        println!("[{}]: {}", i, name);
    });
    println!("[Others]: GSPAT");
    color_print::cprint!("<green><bold>Choose number: ");
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let idx = s
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|&i| i < gains.len())
        .unwrap_or(1);

    let m = Sine::new(150. * Hz);
    let g = gains.swap_remove(idx).1;
    autd.send((m, g))?;

    Ok(true)
}
