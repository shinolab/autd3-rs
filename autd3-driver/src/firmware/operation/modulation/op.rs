use std::sync::Arc;

use crate::{
    derive::{LoopBehavior, SamplingConfig},
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode, MOD_BUF_SIZE_MAX, MOD_BUF_SIZE_MIN, TRANSITION_MODE_NONE},
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::Device,
};

use super::ModulationControlFlags;

#[repr(C, align(2))]
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
struct ModulationSubseq {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
}

pub struct ModulationOp {
    modulation: Arc<Vec<u8>>,
    sent: usize,
    is_done: bool,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl ModulationOp {
    pub const fn new(
        modulation: Arc<Vec<u8>>,
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            modulation,
            sent: 0,
            is_done: false,
            config,
            loop_behavior,
            segment,
            transition_mode,
        }
    }
}

impl Operation for ModulationOp {
    fn pack(&mut self, _: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let is_first = self.sent == 0;

        let offset = if is_first {
            std::mem::size_of::<ModulationHead>()
        } else {
            std::mem::size_of::<ModulationSubseq>()
        };

        let max_mod_size = tx.len() - offset;
        let send_num = if is_first {
            (self.modulation.len() - self.sent)
                .min(max_mod_size)
                .min(254)
        } else {
            (self.modulation.len() - self.sent).min(max_mod_size)
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.modulation.as_ptr().add(self.sent),
                tx.as_mut_ptr().add(offset),
                send_num,
            );
        }

        self.sent += send_num;
        if self.sent > MOD_BUF_SIZE_MAX {
            return Err(AUTDInternalError::ModulationSizeOutOfRange(self.sent));
        }

        let mut flag = if self.segment == Segment::S1 {
            ModulationControlFlags::SEGMENT
        } else {
            ModulationControlFlags::NONE
        };

        if self.modulation.len() == self.sent {
            if self.sent < MOD_BUF_SIZE_MIN {
                return Err(AUTDInternalError::ModulationSizeOutOfRange(self.sent));
            }
            self.is_done = true;
            flag.set(ModulationControlFlags::END, true);
            flag.set(
                ModulationControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        if is_first {
            write_to_tx(
                ModulationHead {
                    tag: TypeTag::Modulation,
                    flag: ModulationControlFlags::BEGIN | flag,
                    size: send_num as _,
                    freq_div: self.config.division()?,
                    rep: self.loop_behavior.rep(),
                    transition_mode: self
                        .transition_mode
                        .map(|m| m.mode())
                        .unwrap_or(TRANSITION_MODE_NONE),
                    transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
                },
                tx,
            );
            Ok(std::mem::size_of::<ModulationHead>() + send_num)
        } else {
            write_to_tx(
                ModulationSubseq {
                    tag: TypeTag::Modulation,
                    flag,
                    size: send_num as _,
                },
                tx,
            );
            Ok(std::mem::size_of::<ModulationSubseq>() + send_num)
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.sent == 0 {
            std::mem::size_of::<ModulationHead>() + 2
        } else {
            std::mem::size_of::<ModulationSubseq>() + 2
        }
    }

    fn is_done(&self) -> bool {
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::{
        mem::{offset_of, size_of},
        num::NonZeroU16,
    };

    use rand::prelude::*;

    use super::*;
    use crate::{
        derive::LoopBehavior, ethercat::DcSysTime, firmware::operation::tests::parse_tx_as,
        geometry::tests::create_device,
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        const MOD_SIZE: usize = 100;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; size_of::<ModulationHead>() + MOD_SIZE];

        let mut rng = rand::thread_rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.gen()).collect();
        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let loop_behavior = LoopBehavior::infinite();
        let rep = loop_behavior.rep;
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
            SamplingConfig::Division(NonZeroU16::new(freq_div).unwrap()),
            LoopBehavior { rep },
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

        assert_eq!(
            TypeTag::Modulation as u8,
            tx[offset_of!(ModulationHead, tag)],
        );
        assert_eq!(
            (ModulationControlFlags::BEGIN
                | ModulationControlFlags::END
                | ModulationControlFlags::TRANSITION)
                .bits(),
            tx[offset_of!(ModulationHead, flag)]
        );
        assert_eq!(
            MOD_SIZE as u8,
            parse_tx_as::<u8>(&tx[offset_of!(ModulationHead, size)..])
        );
        assert_eq!(
            freq_div,
            parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, freq_div)..])
        );
        assert_eq!(
            rep,
            parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, rep)..])
        );
        assert_eq!(
            transition_mode.mode(),
            tx[offset_of!(ModulationHead, transition_mode)]
        );
        assert_eq!(
            transition_mode.value(),
            parse_tx_as::<u64>(&tx[offset_of!(ModulationHead, transition_value)..])
        );

        tx.iter()
            .skip(std::mem::size_of::<ModulationHead>())
            .zip(buf.iter())
            .for_each(|(d, m)| {
                assert_eq!(d, m);
            })
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - std::mem::size_of::<ModulationHead>()
            + (FRAME_SIZE - std::mem::size_of::<ModulationSubseq>()) * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.gen()).collect();

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::FREQ_40K,
            LoopBehavior::infinite(),
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                std::mem::size_of::<ModulationHead>() + 2
            );
            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

            assert_eq!(op.sent, FRAME_SIZE - std::mem::size_of::<ModulationHead>());

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(
                ModulationControlFlags::BEGIN.bits(),
                tx[offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationHead>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, size)..])
            );
            tx.iter()
                .skip(std::mem::size_of::<ModulationHead>())
                .zip(buf.iter().take(mod_size))
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                std::mem::size_of::<ModulationSubseq>() + 2
            );

            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

            assert_eq!(
                op.sent,
                FRAME_SIZE - std::mem::size_of::<ModulationHead>() + FRAME_SIZE
                    - std::mem::size_of::<ModulationSubseq>()
            );

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(
                ModulationControlFlags::NONE.bits(),
                tx[offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, size)..])
            );
            tx.iter()
                .skip(std::mem::size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(FRAME_SIZE - std::mem::size_of::<ModulationHead>())
                        .take(mod_size),
                )
                .for_each(|(d, m)| {
                    assert_eq!(d, m);
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                std::mem::size_of::<ModulationSubseq>() + 2
            );

            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));
            assert_eq!(op.sent, MOD_SIZE);

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(
                (ModulationControlFlags::TRANSITION | ModulationControlFlags::END).bits(),
                tx[offset_of!(ModulationHead, flag)]
            );
            let mod_size = FRAME_SIZE - std::mem::size_of::<ModulationSubseq>();
            assert_eq!(
                mod_size as u16,
                parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, size)..])
            );
            tx.iter()
                .skip(std::mem::size_of::<ModulationSubseq>())
                .zip(
                    buf.iter()
                        .skip(
                            FRAME_SIZE - std::mem::size_of::<ModulationHead>() + FRAME_SIZE
                                - std::mem::size_of::<ModulationSubseq>(),
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
    #[case(Err(AUTDInternalError::ModulationSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDInternalError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MIN-1)), MOD_BUF_SIZE_MIN-1)]
    #[case(Ok(()), MOD_BUF_SIZE_MIN)]
    #[case(Ok(()), MOD_BUF_SIZE_MAX)]
    #[case(
        Err(AUTDInternalError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MAX+1)),
        MOD_BUF_SIZE_MAX+1
    )]
    #[cfg_attr(miri, ignore)]
    fn out_of_range(#[case] expected: Result<(), AUTDInternalError>, #[case] size: usize) {
        let send = |n: usize| {
            const FRAME_SIZE: usize = size_of::<ModulationHead>() + NUM_TRANS_IN_UNIT * 2;
            let device = create_device(0, NUM_TRANS_IN_UNIT);
            let mut tx = vec![0x00u8; FRAME_SIZE];
            let buf = Arc::new(vec![0x00; n]);
            let mut op = ModulationOp::new(
                buf.clone(),
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
                Segment::S0,
                None,
            );
            loop {
                op.pack(&device, &mut tx)?;
                if op.is_done() {
                    break;
                }
            }
            Result::<(), AUTDInternalError>::Ok(())
        };
        assert_eq!(expected, send(size));
    }
}
