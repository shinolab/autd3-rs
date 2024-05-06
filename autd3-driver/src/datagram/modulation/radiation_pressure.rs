use crate::derive::*;

/// Modulation for modulating radiation pressure instead of amplitude
#[derive(Modulation)]
#[no_radiation_pressure]
#[no_modulation_transform]
pub struct RadiationPressure<M: Modulation> {
    m: M,
    #[no_change]
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
}

impl<M: Modulation> RadiationPressure<M> {
    #[doc(hidden)]
    pub fn new(m: M) -> Self {
        Self {
            config: m.sampling_config(),
            loop_behavior: m.loop_behavior(),
            m,
        }
    }
}

pub trait IntoRadiationPressure<M: Modulation> {
    /// Apply modulation to radiation pressure instead of amplitude
    fn with_radiation_pressure(self) -> RadiationPressure<M>;
}

impl<M: Modulation> Modulation for RadiationPressure<M> {
    fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
        Ok(self
            .m
            .calc(geometry)?
            .into_iter()
            .map(|(i, v)| {
                (
                    i,
                    v.into_iter()
                        .map(|v| ((v as f64 / 255.).sqrt() * 255.).round() as u8)
                        .collect(),
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::{super::tests::TestModulation, *};

    use crate::geometry::tests::create_geometry;

    #[rstest::rstest]
    #[test]
    #[case::freq_4k(SamplingConfig::FREQ_4K_HZ)]
    #[case::disable(SamplingConfig::DISABLE)]
    fn test_sampling_config(#[case] config: SamplingConfig) {
        assert_eq!(
            config,
            TestModulation {
                buf: vec![u8::MIN; 2],
                config,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .sampling_config()
        );
    }

    #[test]
    fn test() {
        let geometry = create_geometry(1, 249);

        let mut rng = rand::thread_rng();

        let buf = vec![rng.gen(), rng.gen()];
        assert_eq!(
            Ok(HashMap::from([(
                0,
                buf.iter()
                    .map(|&x| (((x as f64 / 255.).sqrt() * 255.).round() as u8).into())
                    .collect::<Vec<u8>>()
            )])),
            TestModulation {
                buf: buf.clone(),
                config: SamplingConfig::FREQ_4K_HZ,
                loop_behavior: LoopBehavior::infinite(),
            }
            .with_radiation_pressure()
            .calc(&geometry)
        );
    }
}
