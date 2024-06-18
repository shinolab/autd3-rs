pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;

use super::GainCalcResult;

#[derive(Gain)]
#[no_gain_transform]
pub struct Transform<
    G: Gain,
    FT: Fn(&Transducer, Drive) -> Drive + Send + Sync + 'static,
    F: Fn(&Device) -> FT,
> {
    gain: G,
    f: F,
}

pub trait IntoTransform<G: Gain> {
    fn with_transform<
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    >(
        self,
        f: F,
    ) -> Transform<G, FT, F>;
}

impl<
        G: Gain,
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    > Transform<G, FT, F>
{
    #[doc(hidden)]
    pub const fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
}

impl<
        G: Gain,
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync + 'static,
        F: Fn(&Device) -> FT,
    > Gain for Transform<G, FT, F>
{
    fn calc(&self, geometry: &Geometry) -> GainCalcResult {
        let src = self.gain.calc(geometry)?;
        let f = &self.f;
        Ok(Box::new(move |dev| {
            let f = f(dev);
            let src = src(dev);
            Box::new(move |tr| f(tr, src(tr)))
        }))
    }

    #[tracing::instrument(skip(self, geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
        <G as Gain>::trace(&self.gain, geometry);
    }
    // GRCOV_EXCL_STOP
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestGain, *};

    use crate::{defined::FREQ_40K, geometry::tests::create_geometry};

    #[test]
    fn test() {
        let geometry = create_geometry(1, 249, FREQ_40K);

        let mut rng = rand::thread_rng();
        let d: Drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));

        let gain = TestGain::null(&geometry).with_transform(move |_| move |_, _| d);

        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            geometry
                .devices()
                .map(|dev| (
                    dev.idx(),
                    dev.iter().map(gain.calc(&geometry).unwrap()(dev)).collect()
                ))
                .collect()
        );
    }
}
