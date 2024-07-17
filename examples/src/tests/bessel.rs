use autd3::{driver::link::Link, prelude::*};

pub async fn bessel(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let center = autd.geometry().center();
    let dir = Vector3::z();

    let g = Bessel::new(center, dir, 18. / 180. * PI * rad);
    let m = Sine::new(150. * Hz);

    autd.send((m, g)).await?;

    Ok(true)
}
