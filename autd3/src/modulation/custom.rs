use std::collections::HashMap;

use autd3_driver::derive::*;

#[derive(Modulation)]
pub struct Custom<F: Fn(&Device) -> Result<Vec<u8>, AUTDInternalError> + Sync> {
    f: F,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<F: Fn(&Device) -> Result<Vec<u8>, AUTDInternalError> + Sync> Custom<F> {
    /// constructor
    pub const fn new(config: SamplingConfig, f: F) -> Self {
        Self {
            config,
            f,
            loop_behavior: LoopBehavior::infinite(),
        }
    }
}

impl<F: Fn(&Device) -> Result<Vec<u8>, AUTDInternalError> + Sync> Modulation for Custom<F> {
    fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
        Self::transform(geometry, &self.f)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::tests::create_geometry;
    use autd3_driver::{datagram::Datagram, defined::kHz};

    use super::*;

    #[test]
    fn test_custom() -> anyhow::Result<()> {
        let mut rng = rand::thread_rng();

        let geometry = create_geometry(2);

        let test_buf = vec![rng.gen(); 100];
        let custom = Custom::new(SamplingConfig::Freq(4 * kHz), |dev| {
            Ok(if dev.idx() == 0 {
                test_buf.clone()
            } else {
                vec![]
            })
        });

        let res = custom.calc(&geometry)?;
        assert_eq!(res[&0], test_buf);
        assert!(res[&1].is_empty());

        Ok(())
    }

    // GRCOV_EXCL_START
    fn f(_: &Device) -> Result<Vec<u8>, AUTDInternalError> {
        Ok(vec![])
    }
    // GRCOV_EXCL_STOP

    #[test]
    fn test_transtest_derive() {
        let gain = Custom::new(SamplingConfig::Freq(4 * kHz), f);
        let _ = gain.operation();
    }
}
