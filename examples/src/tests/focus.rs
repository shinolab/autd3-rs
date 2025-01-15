use autd3::{core::link::Link, prelude::*};

pub fn focus(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus::new(center);
    let m = Sine::new(150. * Hz);

    autd.send((m, g))?;

    Ok(true)
}
