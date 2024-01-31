use std::collections::HashMap;

use autd3_driver::{
    derive::*,
    geometry::{Device, Geometry},
};

/// Gain to drive only specified transducers
#[derive(Gain)]
pub struct TransducerTest<F: Fn(&Device, &Transducer) -> Option<Drive> + 'static> {
    f: F,
}

impl<F: Fn(&Device, &Transducer) -> Option<Drive> + 'static> TransducerTest<F> {
    /// constructor
    pub const fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&Device, &Transducer) -> Option<Drive> + 'static> Gain for TransducerTest<F> {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |dev, tr| {
            (self.f)(dev, tr).unwrap_or(Drive::null())
        }))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::tests::create_geometry;

    use super::*;

    #[test]
    fn test_transducer_test() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(1);

        let test_id = rng.gen_range(0..geometry.num_transducers());
        let test_drive = Drive::new(Phase::new(rng.gen()), EmitIntensity::new(rng.gen()));
        let transducer_test = TransducerTest::new(move |dev, tr| {
            if (dev.idx() == 0) && (tr.idx() == test_id) {
                Some(test_drive)
            } else {
                None
            }
        });

        let drives = transducer_test.calc(&geometry, GainFilter::All)?;
        drives[&0].iter().enumerate().for_each(|(idx, &drive)| {
            if idx == test_id {
                assert_eq!(test_drive, drive);
            } else {
                assert_eq!(Drive::null(), drive);
            }
        });

        Ok(())
    }

    #[test]
    fn test_transtest_derive() {
        let gain = TransducerTest::new(|_, _| None);
        let _ = gain.operation();
    }
}
