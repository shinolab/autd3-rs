use std::time::Duration;

use crate::{
    common::ULTRASOUND_FREQ,
    error::AUTDDriverError,
    firmware::operation::{Operation, TypeTag},
    geometry::Device,
};

use super::SilencerControlFlags;

use zerocopy::{Immutable, IntoBytes};

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct SilencerFixedCompletionTime {
    tag: TypeTag,
    flag: SilencerControlFlags,
    value_intensity: u16,
    value_phase: u16,
}

pub struct SilencerFixedCompletionTimeOp {
    is_done: bool,
    intensity: Duration,
    phase: Duration,
    strict_mode: bool,
}

impl SilencerFixedCompletionTimeOp {
    pub(crate) const fn new(intensity: Duration, phase: Duration, strict_mode: bool) -> Self {
        Self {
            is_done: false,
            intensity,
            phase,
            strict_mode,
        }
    }
}

impl Operation for SilencerFixedCompletionTimeOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        let validate = |value: Duration| {
            const NANOSEC: u128 = 1_000_000_000;
            let v = value.as_nanos() * ULTRASOUND_FREQ.hz() as u128;
            let v = if v % NANOSEC == 0 {
                v / NANOSEC
            } else {
                return Err(AUTDDriverError::InvalidSilencerCompletionTime(value));
            };
            if v == 0 || v > u16::MAX as u128 {
                return Err(AUTDDriverError::SilencerCompletionTimeOutOfRange(value));
            }
            Ok(v as u16)
        };
        let step_intensity = validate(self.intensity)?;
        let step_phase = validate(self.phase)?;

        super::super::write_to_tx(
            tx,
            SilencerFixedCompletionTime {
                tag: TypeTag::Silencer,
                flag: if self.strict_mode {
                    SilencerControlFlags::STRICT_MODE
                } else {
                    SilencerControlFlags::NONE
                },
                value_intensity: step_intensity,
                value_phase: step_phase,
            },
        );

        self.is_done = true;
        Ok(std::mem::size_of::<SilencerFixedCompletionTime>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<SilencerFixedCompletionTime>()
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::{common::ULTRASOUND_PERIOD, firmware::operation::tests::create_device};

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[rstest::rstest]
    #[test]
    #[case(SilencerControlFlags::STRICT_MODE.bits(), true)]
    #[case(0x00, false)]
    fn test(#[case] value: u8, #[case] strict_mode: bool) {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionTime>()];

        let mut op = SilencerFixedCompletionTimeOp::new(
            ULTRASOUND_PERIOD * 0x12,
            ULTRASOUND_PERIOD * 0x34,
            strict_mode,
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<SilencerFixedCompletionTime>()
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
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(0),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 65536)),
        Duration::from_micros(25 * 65536),
        Duration::from_micros(25)
    )]
    #[case(
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_micros(0)),
        Duration::from_micros(25),
        Duration::from_micros(0)
    )]
    #[case(
        AUTDDriverError::SilencerCompletionTimeOutOfRange(Duration::from_micros(25 * 65536)),
        Duration::from_micros(25),
        Duration::from_micros(25 * 65536),
    )]
    #[case(
        AUTDDriverError::InvalidSilencerCompletionTime(Duration::from_micros(26)),
        Duration::from_micros(26),
        Duration::from_micros(50)
    )]
    #[case(
        AUTDDriverError::InvalidSilencerCompletionTime(Duration::from_micros(51)),
        Duration::from_micros(25),
        Duration::from_micros(51)
    )]
    fn invalid_time(
        #[case] expected: AUTDDriverError,
        #[case] time_intensity: Duration,
        #[case] time_phase: Duration,
    ) {
        let device = create_device(NUM_TRANS_IN_UNIT);

        let mut tx = [0x00u8; size_of::<SilencerFixedCompletionTime>()];

        let mut op = SilencerFixedCompletionTimeOp::new(time_intensity, time_phase, true);

        assert_eq!(expected, op.pack(&device, &mut tx).unwrap_err());
    }
}
