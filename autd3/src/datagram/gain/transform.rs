pub use crate::driver::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::{Device, Geometry, Transducer},
};

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

impl<
        G: Gain,
        D: Into<Drive>,
        FT: Fn(&Transducer, Drive) -> D + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    > Transform<G, D, FT, F>
{
    const fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
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
    > Gain for Transform<G, D, FT, F>
{
    fn calc(&self, geometry: &Geometry) -> Result<GainCalcFn, AUTDInternalError> {
        let mut src = self.gain.calc(geometry)?;
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
    use crate::{gain::Null, tests::create_geometry};

    use super::*;

    use autd3_driver::firmware::fpga::{EmitIntensity, Phase};
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn test() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let mut rng = rand::thread_rng();
        let d = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let gain = Null::new().with_transform(move |_| move |_, _| d);

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
