use std::time::Duration;

use crate::{
    defined::ULTRASOUND_FREQ,
    error::AUTDInternalError,
    firmware::{
        fpga::SilencerTarget,
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::Device,
};

use super::{SILENCER_FLAG_PULSE_WIDTH, SILENCER_FLAG_STRICT_MODE};

#[repr(C, align(2))]
struct SilencerFixedCompletionSteps {
    tag: TypeTag,
    flag: u8,
    value_intensity: u16,
    value_phase: u16,
}

pub struct SilencerFixedCompletionStepsOp {
    is_done: bool,
    value_intensity: Duration,
    value_phase: Duration,
    strict_mode: bool,
    target: SilencerTarget,
}

impl SilencerFixedCompletionStepsOp {
    pub const fn new(
        value_intensity: Duration,
        value_phase: Duration,
        strict_mode: bool,
        target: SilencerTarget,
    ) -> Self {
        Self {
            is_done: false,
            value_intensity,
            value_phase,
            strict_mode,
            target,
        }
    }
}

impl Operation for SilencerFixedCompletionStepsOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let validate = |value: Duration| {
            const NANOSEC: u128 = 1_000_000_000;
            let v = value.as_nanos() * ULTRASOUND_FREQ.hz() as u128;
            let v = if v % NANOSEC == 0 {
                v / NANOSEC
            } else {
                return Err(AUTDInternalError::InvalidSilencerCompletionTime(value));
            };
            if v == 0 || v > u8::MAX as _ {
                return Err(AUTDInternalError::SilencerCompletionTimeOutOfRange(value));
            }
            Ok(v as u16)
        };
        let step_intensity = validate(self.value_intensity)?;
        let step_phase = validate(self.value_phase)?;

        write_to_tx(
            SilencerFixedCompletionSteps {
                tag: TypeTag::Silencer,
                flag: if self.strict_mode {
                    SILENCER_FLAG_STRICT_MODE
                } else {
                    0
                } | match self.target {
                    SilencerTarget::Intensity => 0,
                    SilencerTarget::PulseWidth => SILENCER_FLAG_PULSE_WIDTH,
                },
                value_intensity: step_intensity,
                value_phase: step_phase,
            },
            tx,
        );

        self.is_done = true;
        Ok(std::mem::size_of::<SilencerFixedCompletionSteps>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SilencerFixedCompletionSteps>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::{
        defined::ULTRASOUND_PERIOD, firmware::fpga::SilencerTarget, geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[rstest::rstest]
    #[test]
    #[case(SILENCER_FLAG_STRICT_MODE, true)]
    #[cfg_attr(miri, ignore)]
    #[case(0x00, false)]
    fn test(#[case] value: u8, #[case] strict_mode: bool) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionSteps>()];

        let mut op = SilencerFixedCompletionStepsOp::new(
            ULTRASOUND_PERIOD * 0x12,
            ULTRASOUND_PERIOD * 0x34,
            strict_mode,
            SilencerTarget::Intensity,
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedCompletionSteps>()
        );
        assert!(!op.is_done());

        assert!(op.pack(&device, &mut tx).is_ok());

        assert!(op.is_done());

        assert_eq!(tx[0], TypeTag::Silencer as u8);
        assert_eq!(tx[1], value);
        assert_eq!(tx[2], 0x12);
        assert_eq!(tx[3], 0x00);
        assert_eq!(tx[4], 0x34);
        assert_eq!(tx[5], 0x00);
    }

    #[rstest::rstest]
    #[test]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(0),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 256)),
        Duration::from_micros(25 * 256),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(25),
        Duration::from_micros(0)
    )]
    #[case(
        AUTDInternalError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 256)),
        Duration::from_micros(25),
        Duration::from_micros(25 * 256),
    )]
    #[case(
        AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(26)),
        Duration::from_micros(26),
        Duration::from_micros(50)
    )]
    #[case(
        AUTDInternalError::InvalidSilencerCompletionTime(Duration::from_micros(51)),
        Duration::from_micros(25),
        Duration::from_micros(51)
    )]
    #[cfg_attr(miri, ignore)]
    fn invalid_time(
        #[case] expected: AUTDInternalError,
        #[case] time_intensity: Duration,
        #[case] time_phase: Duration,
    ) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionSteps>()];

        let mut op = SilencerFixedCompletionStepsOp::new(
            time_intensity,
            time_phase,
            true,
            SilencerTarget::Intensity,
        );

        assert_eq!(expected, op.pack(&device, &mut tx).unwrap_err());
    }
}
