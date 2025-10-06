use autd3::{core::link::Link, prelude::*};

pub fn foci_stm(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::disable())?;

    let stm = FociSTM {
        foci: Circle {
            center: autd.center() + Vector3::new(0., 0., 150.0 * mm),
            radius: 30.0 * mm,
            num_points: 50,
            n: Vector3::z_axis(),
            intensity: Intensity::MAX,
        },
        config: 1.0 * Hz,
    };

    let m = Static::default();

    autd.send((m, stm))?;

    Ok(())
}

pub fn gain_stm(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::disable())?;

    let stm = GainSTM {
        gains: Circle {
            center: autd.center() + Vector3::new(0., 0., 150.0 * mm),
            radius: 30.0 * mm,
            num_points: 50,
            n: Vector3::z_axis(),
            intensity: Intensity::MAX,
        },
        config: 1.0 * Hz,
        option: Default::default(),
    };

    let m = Static::default();

    autd.send((m, stm))?;

    Ok(())
}
