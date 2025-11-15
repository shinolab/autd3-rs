use autd3::{core::link::Link, prelude::*};

pub fn plane(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::default())?;

    let g = Plane {
        dir: Vector3::z_axis(),
        option: Default::default(),
    };

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(())
}
