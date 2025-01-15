use autd3::{core::link::Link, prelude::*};

pub fn custom(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable())?;

    let m = autd3::modulation::Custom::new([0, 255], 4 * kHz)?;
    let g = autd3::gain::Custom::new(|dev| {
        let dev_idx = dev.idx();
        move |tr| match (dev_idx, tr.idx()) {
            (0, 0) | (0, 248) => EmitIntensity::MAX,
            _ => EmitIntensity::MIN,
        }
    });

    autd.send((m, g))?;

    Ok(true)
}
