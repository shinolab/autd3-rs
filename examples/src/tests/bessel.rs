use autd3::{core::link::Link, prelude::*};

pub fn bessel(
    autd: &mut Controller<impl Link, firmware::Auto>,
) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Silencer::default())?;

    let center = autd.center();
    let dir = Vector3::z_axis();

    let g = Bessel {
        pos: center,
        dir,
        theta: 18. / 180. * PI * rad,
        option: Default::default(),
    };
    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(())
}
