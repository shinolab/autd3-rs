use autd3_core::derive::*;
use autd3_driver::geometry::{Device, Transducer};

/// [`Gain`] to use arbitrary phases and intensities
///
/// # Examples
///
/// ```
/// use autd3::prelude::*;
/// use autd3::gain::Custom;
///
/// Custom::new(|dev| |tr| Drive { phase: Phase::ZERO, intensity: Intensity::MAX });
/// ```
#[derive(Gain, Debug)]
pub struct Custom<FT, F> {
    /// The function to calculate the phase and intensity
    pub f: F,
    _phantom: std::marker::PhantomData<FT>,
}

impl<'a, FT: Fn(&'a Transducer) -> Drive + Send + Sync, F: Fn(&'a Device) -> FT> Custom<FT, F> {
    /// Create a new [`Custom`]
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct Impl<'a, FT: Fn(&'a Transducer) -> Drive + Send + Sync> {
    f: FT,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, FT: Fn(&'a Transducer) -> Drive + Send + Sync> GainCalculator<'a> for Impl<'a, FT> {
    fn calc(&self, tr: &'a Transducer) -> Drive {
        (self.f)(tr)
    }
}

impl<'a, FT: Fn(&'a Transducer) -> Drive + Send + Sync, F: Fn(&'a Device) -> FT>
    GainCalculatorGenerator<'a> for Custom<FT, F>
{
    type Calculator = Impl<'a, FT>;

    fn generate(&mut self, device: &'a Device) -> Self::Calculator {
        Impl {
            f: (self.f)(device),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, FT: Fn(&'a Transducer) -> Drive + Send + Sync, F: Fn(&'a Device) -> FT> Gain<'a>
    for Custom<FT, F>
{
    type G = Custom<FT, F>;

    fn init(
        self,
        _: &'a Geometry,
        _: &Environment,
        _: &TransducerMask,
    ) -> Result<Self::G, GainError> {
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn custom() {
        let mut rng = rand::rng();

        let geometry = create_geometry(2);
        let env = Environment::new();

        let id = rng.random_range(0..geometry[0].num_transducers());
        let drive = Drive {
            phase: Phase(rng.random()),
            intensity: Intensity(rng.random()),
        };
        let transducer_test = Custom::new(move |dev| {
            move |tr| {
                if dev.idx() == 0 && tr.idx() == id {
                    drive
                } else {
                    Drive::NULL
                }
            }
        });

        let mut d = transducer_test
            .init(&geometry, &env, &TransducerMask::AllEnabled)
            .unwrap();
        geometry.iter().for_each(|dev| {
            let d = d.generate(dev);
            dev.iter().enumerate().for_each(|(idx, tr)| {
                if dev.idx() == 0 && idx == id {
                    assert_eq!(drive, d.calc(tr));
                } else {
                    assert_eq!(Drive::NULL, d.calc(tr));
                }
            });
        });
    }
}
