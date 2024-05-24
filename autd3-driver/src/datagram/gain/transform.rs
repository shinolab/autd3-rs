pub use crate::{
    derive::*,
    error::AUTDInternalError,
    firmware::fpga::{Drive, Segment},
    firmware::operation::{GainOp, NullOp},
    geometry::{Device, Geometry, Transducer},
};
pub use autd3_derive::Gain;

use super::GainCalcFn;

#[derive(Gain)]
#[no_gain_transform]
pub struct Transform<
    G: Gain + 'static,
    FT: Fn(&Transducer, Drive) -> Drive + Send + Sync + 'static,
    F: Fn(&Device) -> FT + Send + Sync + 'static,
> {
    gain: G,
    f: F,
}

pub trait IntoTransform<G: Gain> {
    fn with_transform<
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync,
        F: Fn(&Device) -> FT + Send + Sync,
    >(
        self,
        f: F,
    ) -> Transform<G, FT, F>;
}

impl<
        G: Gain,
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync,
        F: Fn(&Device) -> FT + Send + Sync,
    > Transform<G, FT, F>
{
    #[doc(hidden)]
    pub fn new(gain: G, f: F) -> Self {
        Self { gain, f }
    }
}

impl<
        G: Gain,
        FT: Fn(&Transducer, Drive) -> Drive + Send + Sync,
        F: Fn(&Device) -> FT + Send + Sync,
    > Gain for Transform<G, FT, F>
{
    fn calc<'a>(
        &'a self,
        geometry: &'a Geometry,
        filter: GainFilter<'a>,
    ) -> Result<GainCalcFn<'a>, AUTDInternalError> {
        let src = self.gain.calc(geometry, filter)?;
        Ok(Box::new(move |dev| {
            let f = (self.f)(dev);
            let src = src(dev);
            Box::new(move |tr| f(tr, src(tr)))
        }))
    }
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

        let gain = TestGain::null().with_transform(move |_| move |_, _| d);

        assert_eq!(
            geometry
                .devices()
                .map(|dev| (dev.idx(), vec![d; dev.num_transducers()]))
                .collect::<HashMap<_, _>>(),
            geometry
                .devices()
                .map(|dev| (dev.idx(), {
                    let f = gain.calc(&geometry, GainFilter::All).unwrap()(dev);
                    dev.iter().map(|tr| f(tr)).collect()
                }))
                .collect()
        );
    }
}
