use std::sync::Arc;

use crate::{
    derive::SamplingConfig,
    error::AUTDInternalError,
    firmware::{
        fpga::{Segment, TransitionMode, MOD_BUF_SIZE_MAX, MOD_BUF_SIZE_MIN, TRANSITION_MODE_NONE},
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

use super::ModulationControlFlags;

#[repr(C, align(2))]
struct ModulationHead {
    tag: TypeTag,
    flag: ModulationControlFlags,
    size: u16,
    transition_mode: u8,
    __pad: [u8; 3],
    freq_div: u32,
    rep: u32,
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
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl ModulationOp {
    pub fn new(
        modulation: Arc<Vec<u8>>,
        config: SamplingConfig,
        rep: u32,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            modulation,
            sent: 0,
            is_done: false,
            config,
            rep,
            segment,
            transition_mode,
        }
    }
}

impl Operation for ModulationOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let is_first = self.sent == 0;

        let offset = if is_first {
            std::mem::size_of::<ModulationHead>()
        } else {
            std::mem::size_of::<ModulationSubseq>()
        };

        let max_mod_size = tx.len() - offset;
        let send_num = (self.modulation.len() - self.sent).min(max_mod_size);
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.modulation.as_ptr().add(self.sent),
                tx.as_mut_ptr().add(offset),
                send_num,
            );
        }
        // .filter_map(|_| self.modulation.next())
        // .zip(tx[offset..].iter_mut())
        // .map(|(v, dst)| *dst = v)
        // .fold(0, |acc, _| acc + 1);

        self.sent += send_num;
        if self.sent > MOD_BUF_SIZE_MAX {
            return Err(AUTDInternalError::ModulationSizeOutOfRange(self.sent));
        }

        if is_first {
            *cast::<ModulationHead>(tx) = ModulationHead {
                tag: TypeTag::Modulation,
                flag: ModulationControlFlags::BEGIN
                    | if self.segment == Segment::S1 {
                        ModulationControlFlags::SEGMENT
                    } else {
                        ModulationControlFlags::NONE
                    },
                size: send_num as _,
                __pad: [0; 3],
                freq_div: self.config.division(device.ultrasound_freq())?,
                rep: self.rep,
                transition_mode: self
                    .transition_mode
                    .map(|m| m.mode())
                    .unwrap_or(TRANSITION_MODE_NONE),
                transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
            };
        } else {
            *cast::<ModulationSubseq>(tx) = ModulationSubseq {
                tag: TypeTag::Modulation,
                flag: if self.segment == Segment::S1 {
                    ModulationControlFlags::SEGMENT
                } else {
                    ModulationControlFlags::NONE
                },
                size: send_num as _,
            };
        }

        if self.modulation.len() == self.sent {
            if self.sent < MOD_BUF_SIZE_MIN {
                return Err(AUTDInternalError::ModulationSizeOutOfRange(self.sent));
            }
            self.is_done = true;
            let d = cast::<ModulationSubseq>(tx);
            d.flag.set(ModulationControlFlags::END, true);
            d.flag.set(
                ModulationControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        if is_first {
            Ok(std::mem::size_of::<ModulationHead>() + send_num)
        } else {
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
    use std::mem::{offset_of, size_of};

    use rand::prelude::*;

    use super::*;
    use crate::{
        derive::LoopBehavior,
        ethercat::DcSysTime,
        firmware::{
            fpga::{SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN},
            operation::tests::parse_tx_as,
        },
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
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
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
            buf.clone(),
            SamplingConfig::DivisionRaw(freq_div),
            rep,
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
            MOD_SIZE as u16,
            parse_tx_as::<u16>(&tx[offset_of!(ModulationHead, size)..])
        );
        assert_eq!(
            freq_div,
            parse_tx_as::<u32>(&tx[offset_of!(ModulationHead, freq_div)..])
        );
        assert_eq!(
            rep,
            parse_tx_as::<u32>(&tx[offset_of!(ModulationHead, rep)..])
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
            buf.clone(),
            SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN),
            0xFFFFFFFF,
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
}
