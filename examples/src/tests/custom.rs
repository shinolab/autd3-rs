use autd3::{core::link::Link, prelude::*};

pub fn custom(
    autd: &mut Controller<impl Link, firmware::Auto>,
) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::disable())?;

    let m = autd3::modulation::Custom {
        buffer: vec![0, 255],
        sampling_config: 4. * kHz,
    };
    let g = autd3::gain::Custom::new(|dev| {
        move |tr| match (dev.idx(), tr.idx()) {
            (0, 0) | (0, 248) => Drive {
                intensity: Intensity::MAX,
                phase: Phase::ZERO,
            },
            _ => Drive::NULL,
        }
    });

    autd.send((m, g))?;

    Ok(())
}
