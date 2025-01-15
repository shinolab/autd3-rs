use autd3::{core::link::Link, prelude::*};

pub fn foci_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable())?;

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

    autd.send((m, stm))?;

    Ok(true)
}

pub fn gain_stm(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::disable())?;

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

    autd.send((m, stm))?;

    Ok(true)
}
