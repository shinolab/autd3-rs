use std::collections::HashMap;

use autd3::{core::link::Link, prelude::*};

pub fn group_by_device(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    use autd3::datagram::IntoBoxedDatagram;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    autd.group_send(
        |dev| match dev.idx() {
            0 => Some("null"),
            1 => Some("focus"),
            _ => None,
        },
        HashMap::from([
            ("null", Null {}.into_boxed()),
            (
                "focus",
                Focus {
                    pos: center,
                    option: Default::default(),
                }
                .into_boxed(),
            ),
        ]),
    )?;

    Ok(true)
}

pub fn group_by_transducer(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    use autd3::gain::IntoBoxedGain;

    let pos = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Group {
        key_map: move |dev| {
            let cx = dev.center().x;
            move |tr| {
                if tr.position().x < cx {
                    Some("focus")
                } else {
                    Some("null")
                }
            }
        },
        gain_map: HashMap::from([
            (
                "focus",
                Focus {
                    pos,
                    option: Default::default(),
                }
                .into_boxed(),
            ),
            ("null", Null {}.into_boxed()),
        ]),
    };

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };
    autd.send((m, g))?;

    Ok(true)
}
