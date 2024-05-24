pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp, Operation},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;

use std::collections::HashMap;

/// Gain to transform gain data
#[derive(Gain)]
#[no_gain_transform]
pub struct Transform<
    G: Gain + 'static,
    FT: Fn(&Transducer, Drive) -> Drive + 'static,
    F: Fn(&Device) -> FT + 'static,
> {
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
    fn with_transform<FT: Fn(&Transducer, Drive) -> Drive, F: Fn(&Device) -> FT>(
        self,
        f: F,
    ) -> Transform<G, FT, F>;
}

impl<G: Gain, FT: Fn(&Transducer, Drive) -> Drive, F: Fn(&Device) -> FT> Transform<G, FT, F> {
    #[doc(hidden)]
    pub fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
}

impl<G: Gain, FT: Fn(&Transducer, Drive) -> Drive, F: Fn(&Device) -> FT> Gain
    for Transform<G, FT, F>
{
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        let src = self.gain.calc(geometry, filter)?;
        Ok(geometry
            .devices()
            .map(|dev| {
                let f = (self.f)(dev);
                (
                    dev.idx(),
                    dev.iter()
                        .map(|tr| f(tr, src[&dev.idx()][tr.idx()]))
                        .collect(),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();
        let d: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let gain = TestGain::null().with_transform(move |_| move |_, _| d);

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
