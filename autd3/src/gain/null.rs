use std::collections::HashMap;

use autd3_driver::derive::*;

/// Gain to output nothing
#[derive(Gain, Default, Clone, PartialEq, Eq, Debug)]
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
    use crate::tests::create_geometry;
    use autd3_driver::datagram::Datagram;

    use super::*;

    #[test]
    fn test_null() -> anyhow::Result<()> {
        let geometry = create_geometry(1);

        let d = Null::new().calc(&geometry, GainFilter::All)?;
        assert_eq!(geometry.num_devices(), d.len());
        d.iter().for_each(|(&idx, d)| {
            assert_eq!(geometry[idx].num_transducers(), d.len());
            d.iter().for_each(|&d| {
                assert_eq!(Drive::null(), d);
            })
        });

        Ok(())
    }

    #[test]
    fn test_null_derive() {
        let gain = Null::default();
        let _ = gain.clone();
        let _ = gain.operation();
    }
}
