use std::collections::HashMap;

use autd3_driver::{derive::*, geometry::Geometry};

/// Gain to output nothing
#[derive(Gain, Default, Clone, Copy)]
pub struct Null {}

impl Null {
    /// constructor
    pub const fn new() -> Self {
        Self {}
    }
}

impl Gain for Null {
    fn calc(
        &self,
        geometry: &Geometry,
        filter: GainFilter,
    ) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
        Ok(Self::transform(geometry, filter, |_, _| Drive::null()))
    }
}

#[cfg(test)]
mod tests {

    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{IntoDevice, Vector3},
    };

    use super::*;

    #[test]
    fn test_null() {
        let geometry: Geometry = Geometry::new(vec![AUTD3::new(Vector3::zeros()).into_device(0)]);

        let null_gain = Null::new();

        let drives = null_gain.calc(&geometry, GainFilter::All).unwrap();
        assert_eq!(drives.len(), 1);
        assert_eq!(drives[&0].len(), geometry.num_transducers());
        drives[&0].iter().for_each(|d| {
            assert_eq!(d.intensity.value(), 0);
            assert_eq!(d.phase.value(), 0);
        });
    }

    #[test]
    fn test_null_derive() {
        let gain = Null::default();
        let _ = gain.clone();
        let _ = gain.operation();
    }
}
