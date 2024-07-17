use autd3::{driver::link::Link, prelude::*};

pub async fn foci_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);

    let point_num = 50;
    let radius = 30.0 * mm;
    let stm = FociSTM::from_freq(
        1.0 * Hz,
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f32 / point_num as f32;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            center + p
        }),
    )?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}

pub async fn gain_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);

    let point_num = 50;
    let radius = 30.0 * mm;
    let stm = GainSTM::from_freq(
        1.0 * Hz,
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f32 / point_num as f32;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            Focus::new(center + p)
        }),
    )?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}
