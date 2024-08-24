pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;

use super::GainCalcResult;
use derive_more::Debug;

#[derive(Gain, Debug)]
pub struct Transform<
    G: Gain,
    D: Into<Drive>,
    FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
    F: Fn(&Device) -> FT,
> {
    gain: G,
    #[debug(ignore)]
    f: F,
}

pub trait IntoTransform<G: Gain> {
    fn with_transform<
        D: Into<Drive>,
        FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    >(
        self,
        f: F,
    ) -> Transform<G, D, FT, F>;
}

impl<G: Gain> IntoTransform<G> for G {
    fn with_transform<
        D: Into<Drive>,
        FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    >(
        self,
        f: F,
    ) -> Transform<G, D, FT, F> {
        Transform::new(self, f)
    }
}

impl<
        G: Gain,
        D: Into<Drive>,
        FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    > Transform<G, D, FT, F>
{
    #[doc(hidden)]
    pub const fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
}

impl<
        G: Gain,
        D: Into<Drive>,
        FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    > Gain for Transform<G, D, FT, F>
{
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        let src = self.gain.calc(geometry)?;
        let f = &self.f;
        Ok(Box::new(move |dev| {
            let f = f(dev);
            let src = src(dev);
            Box::new(move |tr| f(tr, src(tr)).into())
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::{
        firmware::fpga::{EmitIntensity, Phase},
        geometry::tests::create_geometry,
    };

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();
        let d = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let gain = TestGain::null(&geometry).with_transform(move |_| move |_, _| d);

        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            geometry
                .devices()
                .map(|dev| Ok((
                    dev.idx(),
                    dev.iter().map(gain.calc(&geometry)?(dev)).collect()
                )))
                .collect::<Result<_, AUTDInternalError>>()?
        );

        Ok(())
    }
}
