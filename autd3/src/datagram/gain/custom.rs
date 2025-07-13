use autd3_core::derive::*;
use autd3_driver::geometry::{Device, Transducer};

use derive_more::Debug;

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
#[debug("Custom (Gain)")]
pub struct Custom<FT, F> {
    /// The function to calculate the phase and intensity
    pub f: F,
    _phantom: std::marker::PhantomData<FT>,
}

impl<'dev, 'tr, FT: Fn(&'tr Transducer) -> Drive + Send + Sync, F: Fn(&'dev Device) -> FT>
    Custom<FT, F>
{
    /// Create a new [`Custom`]
    #[must_use]
    pub const fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct Impl<'tr, FT: Fn(&'tr Transducer) -> Drive + Send + Sync> {
    f: FT,
    _phantom: std::marker::PhantomData<&'tr ()>,
}

impl<'tr, FT: Fn(&'tr Transducer) -> Drive + Send + Sync> GainCalculator<'tr> for Impl<'tr, FT> {
    fn calc(&self, tr: &'tr Transducer) -> Drive {
        (self.f)(tr)
    }
}

impl<'dev, 'tr, FT: Fn(&'tr Transducer) -> Drive + Send + Sync, F: Fn(&'dev Device) -> FT>
    GainCalculatorGenerator<'dev, 'tr> for Custom<FT, F>
{
    type Calculator = Impl<'tr, FT>;

    fn generate(&mut self, device: &'dev Device) -> Self::Calculator {
        Impl {
            f: (self.f)(device),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'geo, 'dev, 'tr, FT: Fn(&'tr Transducer) -> Drive + Send + Sync, F: Fn(&'dev Device) -> FT>
    Gain<'geo, 'dev, 'tr> for Custom<FT, F>
{
    type G = Custom<FT, F>;

    fn init(
        self,
        _: &'geo Geometry,
        _: &Environment,
        _: &TransducerFilter,
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
    fn test_custom() {
        let mut rng = rand::rng();

        let geometry = create_geometry(2);
        let env = Environment::new();

        let test_id = rng.random_range(0..geometry[0].num_transducers());
        let test_drive = Drive {
            phase: Phase(rng.random()),
            intensity: Intensity(rng.random()),
        };
        let transducer_test = Custom::new(move |dev| {
            move |tr| {
                if dev.idx() == 0 && tr.idx() == test_id {
                    test_drive
                } else {
                    Drive::NULL
                }
            }
        });

        let mut d = transducer_test
            .init(&geometry, &env, &TransducerFilter::all_enabled())
            .unwrap();
        geometry.iter().for_each(|dev| {
            let d = d.generate(dev);
            dev.iter().enumerate().for_each(|(idx, tr)| {
                if dev.idx() == 0 && idx == test_id {
                    assert_eq!(test_drive, d.calc(tr));
                } else {
                    assert_eq!(Drive::NULL, d.calc(tr));
                }
            });
        });
    }
}
