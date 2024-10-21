use autd3::{driver::link::Link, prelude::*};

pub async fn foci_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let num_points = 50;
    let radius = 30.0 * mm;
    let stm = FociSTM::circle(1.0 * Hz, radius, num_points, center)?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}

pub async fn gain_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let num_points = 50;
    let radius = 30.0 * mm;
    let stm = GainSTM::circle(1.0 * Hz, radius, num_points, center)?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}
