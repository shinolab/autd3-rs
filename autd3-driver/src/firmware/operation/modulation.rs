use std::sync::Arc;

use crate::{
    error::AUTDDriverError,
    firmware::{
        fpga::{
            LoopBehavior, SamplingConfig, Segment, TransitionMode, MOD_BUF_SIZE_MAX,
            MOD_BUF_SIZE_MIN, TRANSITION_MODE_NONE,
        },
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoBytes, Immutable)]
#[repr(C)]
pub struct ModulationControlFlags(u8);

bitflags::bitflags! {
    impl ModulationControlFlags : u8 {
        const NONE           = 0;
        const BEGIN          = 1 << 0;
        const END            = 1 << 1;
        const TRANSITION     = 1 << 2;
        const SEGMENT        = 1 << 3;
    }
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct ModulationHead {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u8,
    transition_mode: u8,
    freq_div: u16,
    rep: u16,
    transition_value: u64,
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct ModulationSubseq {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct ModulationOp {
    modulation: Arc<Vec<u8>>,
    #[new(default)]
    sent: usize,
    #[new(default)]
    is_done: bool,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl Operation for ModulationOp {
    type Error = AUTDDriverError;

    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        let is_first = self.sent == 0;

        let offset = if is_first {
            size_of::<ModulationHead>()
        } else {
            size_of::<ModulationSubseq>()
        };

        let max_mod_size = if is_first {
            (tx.len() - offset).min(254)
        } else {
            tx.len() - offset
        };
        let send_num = (self.modulation.len() - self.sent).min(max_mod_size);

        tx[offset..offset + send_num]
            .copy_from_slice(&self.modulation[self.sent..self.sent + send_num]);

        self.sent += send_num;
        if self.sent > MOD_BUF_SIZE_MAX {
            return Err(AUTDDriverError::ModulationSizeOutOfRange(self.sent));
        }

        let mut flag = if self.segment == Segment::S1 {
            ModulationControlFlags::SEGMENT
        } else {
            ModulationControlFlags::NONE
        };

        if self.modulation.len() == self.sent {
            if self.sent < MOD_BUF_SIZE_MIN {
                return Err(AUTDDriverError::ModulationSizeOutOfRange(self.sent));
            }
            self.is_done = true;
            flag.set(ModulationControlFlags::END, true);
            flag.set(
                ModulationControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        if is_first {
            super::write_to_tx(
                tx,
                ModulationHead {
                    tag: TypeTag::Modulation,
                    flag: ModulationControlFlags::BEGIN | flag,
                    size: send_num as _,
                    freq_div: self.config.division.get(),
                    rep: self.loop_behavior.rep(),
                    transition_mode: self
                        .transition_mode
                        .map(|m| m.mode())
                        .unwrap_or(TRANSITION_MODE_NONE),
                    transition_value: self.transition_mode.map(TransitionMode::value).unwrap_or(0),
                },
            );
            Ok(size_of::<ModulationHead>() + ((send_num + 0x01) & !0x1))
        } else {
            super::write_to_tx(
                tx,
                ModulationSubseq {
                    tag: TypeTag::Modulation,
                    flag,
                    size: send_num as _,
                },
            );
            Ok(size_of::<ModulationSubseq>() + ((send_num + 0x01) & !0x1))
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.sent == 0 {
            size_of::<ModulationHead>() + 2
        } else {
            size_of::<ModulationSubseq>() + 2
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use rand::prelude::*;

    use super::*;
    use crate::{ethercat::DcSysTime, firmware::operation::tests::create_device};

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        const MOD_SIZE: usize = 100;

        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; size_of::<ModulationHead>() + MOD_SIZE];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.random()).collect();
        let freq_div = rng.random_range(0x0001..=0xFFFF);
        let loop_behavior = LoopBehavior::Infinite;
        let rep = loop_behavior.rep();
        let segment = Segment::S0;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(
                time::macros::datetime!(2000-01-01 0:00 UTC)
                    + std::time::Duration::from_nanos(0x0123456789ABCDEF),
            )
            .unwrap(),
        );

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()).unwrap(),
            loop_behavior,
            segment,
            Some(transition_mode),
        );

        assert_eq!(op.required_size(&device), size_of::<ModulationHead>() + 2);

        assert_eq!(op.sent, 0);

        assert_eq!(
            op.pack(&device, &mut tx),
            Ok(size_of::<ModulationHead>() + MOD_SIZE)
        );

        assert_eq!(op.sent, MOD_SIZE);

        assert_eq!(TypeTag::Modulation as u8, tx[0]);
        assert_eq!(
            (ModulationControlFlags::BEGIN
                | ModulationControlFlags::END
                | ModulationControlFlags::TRANSITION)
                .bits(),
            tx[1]
        );
        assert_eq!(MOD_SIZE as u8, tx[2]);
        assert_eq!(transition_mode.mode(), tx[3]);
        assert_eq!(freq_div as u8, tx[4]);
        assert_eq!((freq_div >> 8) as u8, tx[5]);
        assert_eq!(rep as u8, tx[6]);
        assert_eq!((rep >> 8) as u8, tx[7]);
        assert_eq!(transition_mode.value() as u8, tx[8]);
        assert_eq!((transition_mode.value() >> 8) as u8, tx[9]);
        assert_eq!((transition_mode.value() >> 16) as u8, tx[10]);
        assert_eq!((transition_mode.value() >> 24) as u8, tx[11]);
        assert_eq!((transition_mode.value() >> 32) as u8, tx[12]);
        assert_eq!((transition_mode.value() >> 40) as u8, tx[13]);
        assert_eq!((transition_mode.value() >> 48) as u8, tx[14]);
        assert_eq!((transition_mode.value() >> 56) as u8, tx[15]);
        tx.iter()
            .skip(size_of::<ModulationHead>())
            .zip(buf.iter())
            .for_each(|(d, m)| {
                assert_eq!(d, m);
            })
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - size_of::<ModulationHead>()
            + (FRAME_SIZE - size_of::<ModulationSubseq>()) * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.random()).collect();

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::FREQ_MAX,
            LoopBehavior::Infinite,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(op.required_size(&device), size_of::<ModulationHead>() + 2);
            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

            assert_eq!(op.sent, FRAME_SIZE - size_of::<ModulationHead>());
            let mod_size = FRAME_SIZE - size_of::<ModulationHead>();

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(ModulationControlFlags::BEGIN.bits(), tx[1]);
            assert_eq!(mod_size as u8, tx[2]);

            tx.iter()
                .skip(size_of::<ModulationHead>())
                .zip(buf.iter().take(mod_size))
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
        }

        // Second frame
        {
            assert_eq!(op.required_size(&device), size_of::<ModulationSubseq>() + 2);

            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

            assert_eq!(
                op.sent,
                FRAME_SIZE - size_of::<ModulationHead>() + FRAME_SIZE
                    - size_of::<ModulationSubseq>()
            );

            let mod_size = FRAME_SIZE - size_of::<ModulationSubseq>();

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(ModulationControlFlags::NONE.bits(), tx[1]);
            assert_eq!(mod_size as u8, tx[2]);

            tx.iter()
                .skip(size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(FRAME_SIZE - size_of::<ModulationHead>())
                        .take(mod_size),
                )
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
        }

        // Final frame
        {
            assert_eq!(op.required_size(&device), size_of::<ModulationSubseq>() + 2);

            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));
            assert_eq!(op.sent, MOD_SIZE);

            let mod_size = FRAME_SIZE - size_of::<ModulationSubseq>();

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(
                (ModulationControlFlags::TRANSITION | ModulationControlFlags::END).bits(),
                tx[1]
            );
            assert_eq!(mod_size as u8, tx[2]);

            tx.iter()
                .skip(size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(
                            FRAME_SIZE - size_of::<ModulationHead>() + FRAME_SIZE
                                - size_of::<ModulationSubseq>(),
                        )
                        .take(mod_size),
                )
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
        }
    }

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDDriverError::ModulationSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDDriverError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MIN-1)), MOD_BUF_SIZE_MIN-1)]
    #[case(Ok(()), MOD_BUF_SIZE_MIN)]
    #[case(Ok(()), MOD_BUF_SIZE_MAX)]
    #[case(
        Err(AUTDDriverError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MAX+1)),
        MOD_BUF_SIZE_MAX+1
    )]
    fn out_of_range(#[case] expected: Result<(), AUTDDriverError>, #[case] size: usize) {
        let send = |n: usize| {
            const FRAME_SIZE: usize = size_of::<ModulationHead>() + NUM_TRANS_IN_UNIT * 2;
            let device = create_device(0, NUM_TRANS_IN_UNIT as _);
            let mut tx = vec![0x00u8; FRAME_SIZE];
            let buf = Arc::new(vec![0x00; n]);
            let mut op = ModulationOp::new(
                buf.clone(),
                SamplingConfig::FREQ_MAX,
                LoopBehavior::Infinite,
                Segment::S0,
                None,
            );
            loop {
                op.pack(&device, &mut tx)?;
                if op.is_done() {
                    break;
                }
            }
            Result::<(), AUTDDriverError>::Ok(())
        };
        assert_eq!(expected, send(size));
    }

    #[rstest::rstest]
    #[test]
    #[case(3)]
    #[case(253)]
    #[case(255)]
    fn odd_size(#[case] size: usize) -> anyhow::Result<()> {
        let device = create_device(0, NUM_TRANS_IN_UNIT as _);

        let mut tx = vec![0x00u8; NUM_TRANS_IN_UNIT * 2];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..size).map(|_| rng.random()).collect();

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::FREQ_MAX,
            LoopBehavior::Infinite,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        let mut sent = 0;
        loop {
            let offset = if op.sent == 0 {
                size_of::<ModulationHead>()
            } else {
                size_of::<ModulationSubseq>()
            };
            let size = op.pack(&device, &mut tx)? - offset;
            assert!(size % 2 == 0);
            tx.iter()
                .skip(offset)
                .zip(buf.iter().skip(sent).take(size))
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
            sent += size;
            if op.is_done() {
                break;
            }
        }

        Ok(())
    }
}
