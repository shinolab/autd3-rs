use autd3::{driver::link::Link, prelude::*};

pub async fn focus(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus::new(center);
    let m = Sine::new(150. * Hz);

    autd.send((m, g)).await?;

    Ok(true)
}
