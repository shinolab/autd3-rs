use autd3::prelude::*;

pub async fn focus<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::default()).await?;

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);

    let g = Focus::new(center);
    let m = Sine::new(150.);

    autd.send((m, g)).await?;

    Ok(true)
}
