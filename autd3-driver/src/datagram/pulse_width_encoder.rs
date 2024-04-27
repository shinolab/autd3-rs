use crate::{datagram::*, defined::DEFAULT_TIMEOUT, firmware::fpga::ULTRASOUND_PERIOD};

const PULSE_WIDTH_MAX: u16 = ULTRASOUND_PERIOD as u16 / 2;

#[derive(Debug, Clone)]
pub struct ConfigurePulseWidthEncoder {
    buf: Vec<u16>,
}

impl ConfigurePulseWidthEncoder {
    /// constructor
    pub fn new(buf: Vec<u16>) -> Result<Self, AUTDInternalError> {
        if buf.iter().any(|&v| v > PULSE_WIDTH_MAX)
            || buf
                != buf
                    .iter()
                    .scan(buf[0], |state, &x| {
                        *state = (*state).max(x);
                        Some(*state)
                    })
                    .collect::<Vec<u16>>()
        {
            return Err(AUTDInternalError::InvalidPulseWidthEncoderData);
        }
        Ok(Self { buf })
    }

    pub fn buf(&self) -> &[u16] {
        &self.buf
    }
}

impl Default for ConfigurePulseWidthEncoder {
    fn default() -> Self {
        Self::new(
            include_bytes!("asin.dat")
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    if i >= 0xFF * 0xFF {
                        PULSE_WIDTH_MAX
                    } else {
                        v as u16
                    }
                })
                .collect(),
        )
        .unwrap()
    }
}

impl Datagram for ConfigurePulseWidthEncoder {
    type O1 = crate::firmware::operation::ConfigurePulseWidthEncoderOp;
    type O2 = crate::firmware::operation::NullOp;

    fn operation(self) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
        Ok((Self::O1::new(self.buf), Self::O2::default()))
    }

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::derive::EmitIntensity;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(Ok(vec![0, 0]), vec![0, 0])]
    #[case(Ok(vec![0, PULSE_WIDTH_MAX]), vec![0, PULSE_WIDTH_MAX])]
    #[case(Ok(vec![PULSE_WIDTH_MAX, PULSE_WIDTH_MAX]), vec![PULSE_WIDTH_MAX, PULSE_WIDTH_MAX])]
    #[case(Err(AUTDInternalError::InvalidPulseWidthEncoderData), vec![0, PULSE_WIDTH_MAX + 1])]
    #[case(Err(AUTDInternalError::InvalidPulseWidthEncoderData), vec![PULSE_WIDTH_MAX, 0])]
    fn new(#[case] expected: Result<Vec<u16>, AUTDInternalError>, #[case] buf: Vec<u16>) {
        assert_eq!(
            expected,
            ConfigurePulseWidthEncoder::new(buf).map(|d| d.buf)
        );
    }

    #[test]
    fn default() {
        let datagram = ConfigurePulseWidthEncoder::default();
        assert_eq!(Some(DEFAULT_TIMEOUT), datagram.timeout());
        datagram
            .buf()
            .iter()
            .enumerate()
            .filter(|&(i, _)| {
                i <= EmitIntensity::MAX.value() as usize * EmitIntensity::MAX.value() as usize
            })
            .for_each(|(i, &v)| {
                assert_eq!(
                    ((i as f64
                        / EmitIntensity::MAX.value() as f64
                        / EmitIntensity::MAX.value() as f64)
                        .asin()
                        / PI
                        * PULSE_WIDTH_MAX as f64
                        * 2.)
                        .round() as u16,
                    v
                );
            });
        assert!(datagram.operation().is_ok());
    }
}
