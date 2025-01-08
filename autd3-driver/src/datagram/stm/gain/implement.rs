use std::{collections::HashMap, iter::Peekable};

use bit_vec::BitVec;

use crate::{
    error::AUTDDriverError,
    geometry::{Device, Geometry},
};

use super::{
    gain::GainContext, Gain, GainContextGenerator, GainSTMContext, GainSTMContextGenerator,
    GainSTMGenerator, IntoGainSTMGenerator,
};

pub struct VecGainSTMContext<G: GainContext> {
    gains: Peekable<std::vec::IntoIter<G>>,
}

impl<G: GainContext> GainSTMContext for VecGainSTMContext<G> {
    type Context = G;

    fn next(&mut self) -> Option<Self::Context> {
        self.gains.next()
    }
}

impl<G: GainContextGenerator> GainSTMContextGenerator for Vec<G> {
    type Gain = G;
    type Context = VecGainSTMContext<G::Context>;

    fn generate(&mut self, device: &Device) -> Self::Context {
        Self::Context {
            gains: self
                .iter_mut()
                .map(|g| g.generate(device))
                .collect::<Vec<_>>()
                .into_iter()
                .peekable(),
        }
    }
}

impl<G: Gain> GainSTMGenerator for Vec<G> {
    type T = Vec<G::G>;

    fn init(
        self,
        geometry: &Geometry,
        filter: Option<&HashMap<usize, BitVec<u32>>>,
    ) -> Result<Self::T, AUTDDriverError> {
        self.into_iter()
            .map(|g| g.init(geometry, filter))
            .collect::<Result<Vec<_>, _>>()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl<G: Gain, T> IntoGainSTMGenerator for T
where
    T: IntoIterator<Item = G>,
{
    type G = Vec<G>;

    fn into(self) -> Self::G {
        self.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "dynamic_freq"))]
    use std::time::Duration;

    use super::{super::GainSTM, *};
    use crate::{
        datagram::gain::tests::TestGain,
        defined::{kHz, Freq, Hz},
        firmware::{
            cpu::GainSTMMode,
            fpga::{LoopBehavior, SamplingConfig},
        },
    };

    #[rstest::rstest]
    #[test]
    #[case((1. * Hz).try_into(), 0.5*Hz, 2)]
    #[case((10. * Hz).try_into(), 1.*Hz, 10)]
    #[case((20. * Hz).try_into(), 2.*Hz, 10)]
    #[case((2. * 0.49*Hz).try_into(), 0.49*Hz, 2)]
    fn from_freq(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(freq, (0..n).map(|_| TestGain::null())).map(|g| g.sampling_config())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(SamplingConfig::new_nearest(1. * Hz), 0.5*Hz, 2)]
    #[case(SamplingConfig::new_nearest(0.98 * Hz), 0.49*Hz, 2)]
    #[case(SamplingConfig::new_nearest(10. * Hz), 1.*Hz, 10)]
    #[case(SamplingConfig::new_nearest(20. * Hz), 2.*Hz, 10)]
    fn from_freq_nearest(
        #[case] expect: SamplingConfig,
        #[case] freq: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(freq, (0..n).map(|_| TestGain::null())).sampling_config()
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into(),
        Duration::from_millis(500),
        10
    )]
    #[case(Err(AUTDDriverError::STMPeriodInvalid(2, Duration::from_millis(2000) + Duration::from_nanos(1))), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period(
        #[case] expect: Result<SamplingConfig, AUTDDriverError>,
        #[case] p: Duration,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(p, (0..n).map(|_| TestGain::null())).map(|f| f.sampling_config())
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(
        Duration::from_millis(1000).try_into().unwrap(),
        Duration::from_millis(2000),
        2
    )]
    #[case(
        Duration::from_millis(100).try_into().unwrap(),
        Duration::from_millis(1000),
        10
    )]
    #[case(
        Duration::from_millis(50).try_into().unwrap(),
        Duration::from_millis(500),
        10
    )]
    #[case(Duration::from_millis(1000).try_into().unwrap(), Duration::from_millis(2000) + Duration::from_nanos(1), 2)]
    fn from_period_nearest(#[case] expect: SamplingConfig, #[case] p: Duration, #[case] n: usize) {
        assert_eq!(
            expect,
            GainSTM::new_nearest(p, (0..n).map(|_| TestGain::null())).sampling_config()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case((4. * kHz).try_into().unwrap(), 10)]
    #[case((8. * kHz).try_into().unwrap(), 10)]
    fn from_sampling_config(
        #[case] config: SamplingConfig,
        #[case] n: usize,
    ) -> anyhow::Result<()> {
        assert_eq!(
            Ok(config),
            GainSTM::new(config, (0..n).map(|_| TestGain::null())).map(|f| f.sampling_config())
        );
        Ok(())
    }

    #[rstest::rstest]
    #[test]
    #[case(Ok(0.5*Hz), 0.5*Hz, 2)]
    #[case(Ok(1.0*Hz), 1.*Hz, 10)]
    #[case(Ok(2.0*Hz), 2.*Hz, 10)]
    fn freq(
        #[case] expect: Result<Freq<f32>, AUTDDriverError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| TestGain::null())).map(|f| f.freq())
        );
    }

    #[cfg(not(feature = "dynamic_freq"))]
    #[rstest::rstest]
    #[test]
    #[case(Ok(Duration::from_millis(2000)), 0.5*Hz, 2)]
    #[case(Ok(Duration::from_millis(1000)), 1.*Hz, 10)]
    #[case(Ok(Duration::from_millis(500)), 2.*Hz, 10)]
    fn period(
        #[case] expect: Result<Duration, AUTDDriverError>,
        #[case] f: Freq<f32>,
        #[case] n: usize,
    ) {
        assert_eq!(
            expect,
            GainSTM::new(f, (0..n).map(|_| TestGain::null())).map(|f| f.period())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::phase_intensity_full(GainSTMMode::PhaseIntensityFull)]
    #[case::phase_full(GainSTMMode::PhaseFull)]
    #[case::phase_half(GainSTMMode::PhaseHalf)]
    fn with_mode(#[case] mode: GainSTMMode) {
        assert_eq!(
            mode,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
                .with_mode(mode)
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    fn with_mode_default() {
        assert_eq!(
            GainSTMMode::PhaseIntensityFull,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
                .mode()
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::infinite(LoopBehavior::infinite())]
    #[case::finite(LoopBehavior::once())]
    fn with_loop_behavior(#[case] loop_behavior: LoopBehavior) {
        assert_eq!(
            loop_behavior,
            GainSTM::new(SamplingConfig::FREQ_40K, (0..2).map(|_| TestGain::null()))
                .unwrap()
                .with_loop_behavior(loop_behavior)
                .loop_behavior()
        );
    }
}
