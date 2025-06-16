use std::collections::HashMap;

use autd3::{core::link::Link, prelude::*};

pub fn group_by_device(autd: &mut Controller<impl Link, firmware::Auto>) -> anyhow::Result<bool> {
    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    autd.send(Group {
        key_map: |dev| match dev.idx() {
            0 => Some("null"),
            1 => Some("focus"),
            _ => None,
        },
        datagram_map: HashMap::from([
            ("null", autd.make_boxed(Null {})),
            (
                "focus",
                autd.make_boxed(Focus {
                    pos: center,
                    option: Default::default(),
                }),
            ),
        ]),
    })?;

    Ok(true)
}

pub fn group_by_transducer(
    autd: &mut Controller<impl Link, firmware::Auto>,
) -> anyhow::Result<bool> {
    let pos = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = GainGroup {
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
                BoxedGain::new(Focus {
                    pos,
                    option: Default::default(),
                }),
            ),
            ("null", BoxedGain::new(Null {})),
        ]),
    };

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };
    autd.send((m, g))?;

    Ok(true)
}
