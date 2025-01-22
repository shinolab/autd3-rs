use autd3::{core::link::Link, prelude::*};

pub fn plane(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let dir = Vector3::z_axis();

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };
    let g = Plane {
        dir,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(true)
}
