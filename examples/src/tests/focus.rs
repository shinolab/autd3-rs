use autd3::{core::link::Link, prelude::*};

pub fn focus(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::default())?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus {
        pos: center,
        option: Default::default(),
    };
    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(())
}
