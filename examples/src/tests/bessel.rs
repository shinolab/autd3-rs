use autd3::prelude::*;

pub async fn bessel<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::default()).await?;

    let center = autd.geometry.center();
    let dir = Vector3::z();

    let g = Bessel::new(center, dir, 18. / 180. * PI);
    let m = Sine::new(150.);

    autd.send((m, g)).await?;

    Ok(true)
}
