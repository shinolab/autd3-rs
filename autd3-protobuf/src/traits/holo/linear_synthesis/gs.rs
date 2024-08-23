use autd3_gain_holo::{LinAlgBackend, NalgebraBackend};

use crate::{
    pb::*,
    to_holo,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage
    for autd3_gain_holo::GS<
        autd3_driver::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_driver::acoustics::directivity::Sphere>,
    >
{
    type Message = Datagram;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            datagram: Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Gs(Gs {
                    holo: to_holo!(self),
                    repeat: Some(self.repeat() as _),
                    constraint: Some(self.constraint().to_msg(None)),
                })),
            })),
            parallel_threshold: None,
            timeout: None,
        }
    }
}

impl FromMessage<Gs>
    for autd3_gain_holo::GS<
        autd3_driver::acoustics::directivity::Sphere,
        NalgebraBackend<autd3_driver::acoustics::directivity::Sphere>,
    >
{
    fn from_msg(msg: &Gs) -> Result<Self, AUTDProtoBufError> {
        let mut g = Self::new(
            NalgebraBackend::new()?,
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
        if let Some(repeat) = msg.repeat {
            g = g.with_repeat(repeat as _);
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
    fn test_holo_gs() {
        let mut rng = rand::thread_rng();

        let holo = autd3_gain_holo::GS::new(
            NalgebraBackend::new().unwrap(),
            [
                (
                    Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
                (
                    Vector3::new(rng.gen(), rng.gen(), rng.gen()),
                    rng.gen::<f32>() * autd3_gain_holo::Pa,
                ),
            ],
        )
        .with_repeat(rng.gen());
        let msg = holo.to_msg(None);

        match msg.datagram {
            Some(datagram::Datagram::Gain(Gain {
                gain: Some(gain::Gain::Gs(g)),
                ..
            })) => {
                let holo2 = autd3_gain_holo::GS::from_msg(&g).unwrap();
                assert_eq!(holo.repeat(), holo2.repeat());
                assert_eq!(holo.constraint(), holo2.constraint());
                holo.foci()
                    .iter()
                    .zip(holo2.foci().iter())
                    .for_each(|(f1, f2)| {
                        approx::assert_abs_diff_eq!(f1.x, f2.x);
                        approx::assert_abs_diff_eq!(f1.y, f2.y);
                        approx::assert_abs_diff_eq!(f1.z, f2.z);
                    });
                holo.amps()
                    .iter()
                    .zip(holo2.amps().iter())
                    .for_each(|(f1, f2)| {
                        approx::assert_abs_diff_eq!(f1.pascal(), f2.pascal());
                    });
            }
            _ => panic!("unexpected datagram type"),
        }
    }
}
