use std::collections::HashMap;

use autd3::{core::link::Link, prelude::*};

pub fn group_by_device(autd: &mut Controller<impl Link>) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(Group {
        key_map: |dev| match dev.idx() {
            0 => Some("null"),
            1 => Some("focus"),
            _ => None,
        },
        datagram_map: HashMap::from([
            ("null", BoxedDatagram::new(Null {})),
            (
                "focus",
                BoxedDatagram::new(Focus {
                    pos: autd.center() + Vector3::new(0., 0., 150.0 * mm),
                    option: Default::default(),
                }),
            ),
        ]),
    })?;

    Ok(())
}

pub fn group_by_transducer(
    autd: &mut Controller<impl Link>,
) -> Result<(), Box<dyn std::error::Error>> {
    let g = GainGroup::new(
        move |dev| {
            move |tr| {
                if tr.position().x < dev.center().x {
                    Some("focus")
                } else {
                    Some("null")
                }
            }
        },
        HashMap::from([
            (
                "focus",
                BoxedGain::new(Focus {
                    pos: autd.center() + Vector3::new(0., 0., 150.0 * mm),
                    option: Default::default(),
                }),
            ),
            ("null", BoxedGain::new(Null {})),
        ]),
    );

    let m = Sine {
        freq: 150. * Hz,
        option: Default::default(),
    };

    autd.send((m, g))?;

    Ok(())
}
