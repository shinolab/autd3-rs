use autd3::{core::link::Link, prelude::*};
use autd3_gain_holo::*;

use std::io::{self, Write};

pub fn holo(autd: &mut Controller<impl Link, firmware::Auto>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);
    let p = Vector3::new(30. * mm, 0., 0.);
    let backend = std::sync::Arc::new(NalgebraBackend);
    let target_amp = 2.5e3 * autd.num_devices() as f32 * Pa;
    let foci = [(center + p, target_amp), (center - p, target_amp)];

    let mut gains: Vec<(&str, BoxedGain)> = vec![
        (
            "GS",
            BoxedGain::new(GS {
                foci: foci.to_vec(),
                option: Default::default(),
                backend: backend.clone(),
                directivity: std::marker::PhantomData::<Sphere>,
            }),
        ),
        (
            "GSPAT",
            BoxedGain::new(GSPAT {
                foci: foci.to_vec(),
                option: Default::default(),
                backend: backend.clone(),
                directivity: std::marker::PhantomData::<Sphere>,
            }),
        ),
        (
            "Naive",
            BoxedGain::new(Naive {
                foci: foci.to_vec(),
                option: Default::default(),
                backend: backend.clone(),
                directivity: std::marker::PhantomData::<Sphere>,
            }),
        ),
        (
            "LM",
            BoxedGain::new(LM {
                foci: foci.to_vec(),
                option: Default::default(),
                backend: backend.clone(),
                directivity: std::marker::PhantomData::<Sphere>,
            }),
        ),
        (
            "Greedy",
            BoxedGain::new(Greedy {
                foci: foci.to_vec(),
                option: Default::default(),
                directivity: std::marker::PhantomData::<Sphere>,
            }),
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

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };
    let g = gains.swap_remove(idx).1;
    autd.send((m, g))?;

    Ok(true)
}
