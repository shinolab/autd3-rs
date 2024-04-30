use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_gain_holo::Greedy<autd3_driver::acoustics::directivity::Sphere> {
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(Greedy {
                    holo: to_holo!(self),
                    phase_div: self.phase_div() as _,
                    constraint: Some(self.constraint().to_msg(None)),
                })),
                segment: Segment::S0 as _,
                transition_mode: Some(TransitionMode::SyncIdx.into()),
                transition_value: Some(0),
            })),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegment<
        autd3_gain_holo::Greedy<autd3_driver::acoustics::directivity::Sphere>,
    >
{
    type Message = DatagramLightweight;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(Greedy {
                    holo: to_holo!(self),
                    phase_div: self.phase_div() as _,
                    constraint: Some(self.constraint().to_msg(None)),
                })),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            })),
        }
    }
}

impl FromMessage<Greedy> for autd3_gain_holo::Greedy<autd3_driver::acoustics::directivity::Sphere> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Greedy) -> Option<Self> {
        Some(
            Self::new()
                .with_phase_div(msg.phase_div as _)
                .with_constraint(autd3_gain_holo::EmissionConstraint::from_msg(
                    msg.constraint.as_ref()?,
                )?)
                .add_foci_from_iter(
                    msg.holo
                        .iter()
                        .map(|h| {
                            Some((
                                autd3_driver::geometry::Vector3::from_msg(h.pos.as_ref()?)?,
                                h.amp.as_ref()?.value as f64 * autd3_gain_holo::Pascal,
                            ))
                        })
                        .collect::<Option<Vec<_>>>()?,
                ),
        )
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

        let holo = autd3_gain_holo::Greedy::new()
            .with_phase_div(rng.gen())
            .add_focus(
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<f64>() * autd3_gain_holo::Pascal,
            )
            .add_focus(
                Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                rng.gen::<f64>() * autd3_gain_holo::Pascal,
            );
        let msg = holo.to_msg(None);

        match msg.datagram {
            Some(datagram_lightweight::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Greedy(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::Greedy::from_msg(&g).unwrap();
                assert_eq!(holo.phase_div(), holo2.phase_div());
                assert_eq!(holo.constraint(), holo2.constraint());
                holo.foci().zip(holo2.foci()).for_each(|(f1, f2)| {
                    assert_approx_eq::assert_approx_eq!(f1.0.x, f2.0.x);
                    assert_approx_eq::assert_approx_eq!(f1.0.y, f2.0.y);
                    assert_approx_eq::assert_approx_eq!(f1.0.z, f2.0.z);
                    assert_approx_eq::assert_approx_eq!(f1.1.as_pascal(), f2.1.as_pascal());
                });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
