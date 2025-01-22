use autd3::{core::link::Link, driver::datagram::IntoBoxedGain, prelude::*};

pub fn group_by_device(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    autd.group(|dev| match dev.idx() {
        0 => Some("null"),
        1 => Some("focus"),
        _ => None,
    })
    .set("null", (Static::default(), Null {}))?
    .set(
        "focus",
        (
            Sine {
                freq: 150. * Hz,
                option: Default::default(),
            },
            Focus {
                pos: center,
                option: Default::default(),
            },
        ),
    )?
    .send()?;

    Ok(true)
}

pub fn group_by_transducer(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    let cx = autd.center().x;
    let g1 = Focus {
        pos: autd[0].center() + Vector3::new(0., 0., 150.0 * mm),
        option: Default::default(),
    };
    let g2 = Null {};
    let g = Group::new(move |_dev| {
        move |tr| {
            if tr.position().x < cx {
                Some("focus")
            } else {
                Some("null")
            }
        }
    })
    .set("focus", g1.into_boxed())?
    .set("null", g2.into_boxed())?;

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };
    autd.send((m, g))?;

    Ok(true)
}
