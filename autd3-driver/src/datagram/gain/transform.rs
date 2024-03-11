pub use crate::{
    common::{Drive, Segment},
    datagram::{DatagramS, Gain, GainCache, GainFilter, IntoGainCache, Modulation},
    error::AUTDInternalError,
    geometry::{Device, Geometry, Transducer},
    operation::{GainOp, NullOp, Operation},
};
pub use autd3_derive::Gain;

use std::collections::HashMap;

/// Gain to transform gain data
#[derive(Gain)]
#[no_gain_transform]
pub struct Transform<G: Gain + 'static, F: Fn(&Device, &Transducer, Drive) -> Drive + 'static> {
    gain: G,
    f: F,
}

pub trait IntoTransform<G: Gain> {
    /// transform gain data
    ///
    /// # Arguments
    ///
    /// * `f` - transform function. The first argument is the device, the second is transducer, and the third is the original drive data.
    ///
    fn with_transform<F: Fn(&Device, &Transducer, Drive) -> Drive>(self, f: F) -> Transform<G, F>;
}

impl<G: Gain + 'static, F: Fn(&Device, &Transducer, Drive) -> Drive> Transform<G, F> {
    #[doc(hidden)]
    pub fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
}

impl<G: Gain + 'static, F: Fn(&Device, &Transducer, Drive) -> Drive + 'static> Gain
    for Transform<G, F>
{
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(self
            .gain
            .calc(geometry, filter)?
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .enumerate()
                        .map(|(i, d)| (self.f)(&geometry[k], &geometry[k][i], d))
                        .collect::<Vec<_>>(),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::geometry::tests::create_geometry;

    #[test]
    fn test_gain_transform() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();
        let d: Drive = rng.gen();

        let gain = TestGain::null().with_transform(move |_, _, _| d);

        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            gain.calc(&geometry, GainFilter::All)?
        );

        Ok(())
    }
}
