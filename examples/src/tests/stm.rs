use autd3::{driver::link::AsyncLink, prelude::*};

pub async fn foci_stm(autd: &mut Controller<impl AsyncLink>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let stm = FociSTM::new(
        1.0 * Hz,
        Circle {
            center: autd.center() + Vector3::new(0., 0., 150.0 * mm),
            radius: 30.0 * mm,
            num_points: 50,
            n: Vector3::z_axis(),
            intensity: EmitIntensity::MAX,
        },
    )?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}

pub async fn gain_stm(autd: &mut Controller<impl AsyncLink>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable()).await?;

    let stm = GainSTM::new(
        1.0 * Hz,
        Circle {
            center: autd.center() + Vector3::new(0., 0., 150.0 * mm),
            radius: 30.0 * mm,
            num_points: 50,
            n: Vector3::z_axis(),
            intensity: EmitIntensity::MAX,
        },
    )?;

    let m = Static::new();

    autd.send((m, stm)).await?;

    Ok(true)
}
