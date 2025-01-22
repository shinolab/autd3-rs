use autd3::{core::link::Link, prelude::*};

pub fn custom(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable())?;

    let m = autd3::modulation::Custom {
        buffer: vec![0, 255],
        sampling_config: 4 * kHz,
        option: Default::default(),
    };
    let g = autd3::gain::Custom::new(|dev| {
        let dev_idx = dev.idx();
        move |tr| match (dev_idx, tr.idx()) {
            (0, 0) | (0, 248) => Drive {
                intensity: EmitIntensity::MAX,
                phase: Phase::ZERO,
            },
            _ => Drive::NULL,
        }
    });

    autd.send((m, g))?;

    Ok(true)
}
