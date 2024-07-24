use std::num::NonZeroU8;

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_gain_holo::Greedy<autd3_driver::acoustics::directivity::Sphere> {
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(Greedy {
                    holo: to_holo!(self),
                    phase_div: Some(self.phase_div().get() as _),
                    constraint: Some(self.constraint().to_msg(None)),
                })),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<Greedy> for autd3_gain_holo::Greedy<autd3_driver::acoustics::directivity::Sphere> {
    fn from_msg(msg: &Greedy) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(
            msg.holo
                .iter()
                .map(|h| {
                    Ok((
                        autd3_driver::geometry::Vector3::from_msg(&h.pos)?,
                        autd3_gain_holo::Amplitude::from_msg(&h.amp)?,
                    ))
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
        );
        if let Some(phase_div) = msg.phase_div {
            g = g.with_phase_div(
                NonZeroU8::new(phase_div as u8).ok_or(AUTDProtoBufError::DataParseError)?,
            );
        }
        if let Some(constraint) = msg.constraint.as_ref() {
            g = g.with_constraint(autd3_gain_holo::EmissionConstraint::from_msg(constraint)?);
        }
        Ok(g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::geometry::Vector3;
    use rand::Rng;

    #[test]
    fn test_holo_greedy() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::Greedy::new([
            (
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<f32>() * autd3_gain_holo::Pa,
            ),
            (
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<f32>() * autd3_gain_holo::Pa,
            ),
        ])
        .with_phase_div(rng.gen());
        let msg = holo.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Greedy::from_msg(&g).unwrap();
                assert_eq!(holo.phase_div(), holo2.phase_div());
                assert_eq!(holo.constraint(), holo2.constraint());
                holo.foci()
                    .iter()
                    .zip(holo2.foci().iter())
                    .for_each(|(f1, f2)| {
                        assert_approx_eq::assert_approx_eq!(f1.x, f2.x);
                        assert_approx_eq::assert_approx_eq!(f1.y, f2.y);
                        assert_approx_eq::assert_approx_eq!(f1.z, f2.z);
                    });
                holo.amps()
                    .iter()
                    .zip(holo2.amps().iter())
                    .for_each(|(f1, f2)| {
                        assert_approx_eq::assert_approx_eq!(f1.pascal(), f2.pascal());
                    });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
