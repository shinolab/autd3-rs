use autd3::{driver::link::Link, prelude::*};

pub fn bessel(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let center = autd.center();
    let dir = Vector3::z_axis();

    let g = Bessel::new(center, dir, 18. / 180. * PI * rad);
    let m = Sine::new(150. * Hz);

    autd.send((m, g))?;

    Ok(true)
}
