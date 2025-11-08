use std::sync::Arc;

use crate::{
    error::AUTDDriverError,
    firmware::{
        operation::{Operation, OperationGenerator, implement::null::NullOp},
        tag::TypeTag,
    },
};

use autd3_core::{
    firmware::{
        MOD_BUF_SIZE_MAX, MOD_BUF_SIZE_MIN, SamplingConfig, Segment,
        transition_mode::{Later, TransitionMode, TransitionModeParams},
    },
    geometry::Device,
    modulation::ModulationOperationGenerator,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ModulationControlFlags(u8);

impl ModulationControlFlags {
    const NONE: ModulationControlFlags = ModulationControlFlags(0);
    const BEGIN: ModulationControlFlags = ModulationControlFlags(1 << 0);
    const END: ModulationControlFlags = ModulationControlFlags(1 << 1);
    const TRANSITION: ModulationControlFlags = ModulationControlFlags(1 << 2);
    const SEGMENT: ModulationControlFlags = ModulationControlFlags(1 << 3);

    fn set(&mut self, bit: ModulationControlFlags, value: bool) {
        if value {
            self.0 |= bit.0;
        }
    }
}

impl std::ops::BitOr for ModulationControlFlags {
    type Output = ModulationControlFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        ModulationControlFlags(self.0 | rhs.0)
    }
}

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
    rep: u16,
    segment: Segment,
    transition_params: TransitionModeParams,
}

impl ModulationOp {
    pub(crate) fn new(
        modulation: Arc<Vec<u8>>,
        config: SamplingConfig,
        rep: u16,
        segment: Segment,
        transition_params: TransitionModeParams,
    ) -> Self {
        Self {
            modulation,
            sent: 0,
            is_done: false,
            config,
            rep,
            segment,
            transition_params,
        }
    }
}

impl Operation<'_> for ModulationOp {
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
            return Err(AUTDDriverError::ModulationSizeOutOfRange(
                self.modulation.len(),
            ));
        }

        let mut flag = if self.segment == Segment::S1 {
            ModulationControlFlags::SEGMENT
        } else {
            ModulationControlFlags::NONE
        };

        if self.modulation.len() == self.sent {
            if self.sent < MOD_BUF_SIZE_MIN {
                return Err(AUTDDriverError::ModulationSizeOutOfRange(
                    self.modulation.len(),
                ));
            }
            self.is_done = true;
            flag.set(ModulationControlFlags::END, true);
            flag.set(
                ModulationControlFlags::TRANSITION,
                self.transition_params != Later.params(),
            );
        }

        if is_first {
            crate::firmware::operation::write_to_tx(
                tx,
                ModulationHead {
                    tag: TypeTag::Modulation,
                    flag: ModulationControlFlags::BEGIN | flag,
                    size: send_num as _,
                    freq_div: self.config.divide()?,
                    rep: self.rep,
                    transition_mode: self.transition_params.mode,
                    transition_value: self.transition_params.value,
                },
            );
            Ok(size_of::<ModulationHead>() + ((send_num + 0x01) & !0x1))
        } else {
            crate::firmware::operation::write_to_tx(
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

impl OperationGenerator<'_> for ModulationOperationGenerator {
    type O1 = ModulationOp;
    type O2 = NullOp;

    fn generate(&mut self, _: &Device) -> Option<(Self::O1, Self::O2)> {
        let d = self.g.clone();
        Some((
            Self::O1::new(
                d,
                self.config,
                self.rep,
                self.segment,
                self.transition_params,
            ),
            Self::O2 {},
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::*;

    use autd3_core::{derive::transition_mode, ethercat::DcSysTime};
    use rand::prelude::*;

    #[test]
    fn op() {
        const MOD_SIZE: usize = 100;

        let device = crate::tests::create_device();

        let mut tx = vec![0x00u8; size_of::<ModulationHead>() + MOD_SIZE];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.random()).collect();
        let freq_div = rng.random_range(0x0001..=0xFFFF);
        let rep = rng.random_range(0x0000..0xFFFF);
        let segment = Segment::S0;
        let transition_mode = transition_mode::SysTime(
            DcSysTime::ZERO + std::time::Duration::from_nanos(0x0123456789ABCDEF),
        );

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::new(NonZeroU16::new(freq_div).unwrap()),
            rep,
            segment,
            transition_mode.params(),
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
                .0,
            tx[1]
        );
        assert_eq!(MOD_SIZE as u8, tx[2]);
        assert_eq!(transition_mode.params().mode, tx[3]);
        assert_eq!(freq_div as u8, tx[4]);
        assert_eq!((freq_div >> 8) as u8, tx[5]);
        assert_eq!(rep as u8, tx[6]);
        assert_eq!((rep >> 8) as u8, tx[7]);
        assert_eq!(transition_mode.params().value as u8, tx[8]);
        assert_eq!((transition_mode.params().value >> 8) as u8, tx[9]);
        assert_eq!((transition_mode.params().value >> 16) as u8, tx[10]);
        assert_eq!((transition_mode.params().value >> 24) as u8, tx[11]);
        assert_eq!((transition_mode.params().value >> 32) as u8, tx[12]);
        assert_eq!((transition_mode.params().value >> 40) as u8, tx[13]);
        assert_eq!((transition_mode.params().value >> 48) as u8, tx[14]);
        assert_eq!((transition_mode.params().value >> 56) as u8, tx[15]);
        tx.iter()
            .skip(size_of::<ModulationHead>())
            .zip(buf.iter())
            .for_each(|(d, m)| {
                assert_eq!(d, m);
            })
    }

    #[test]
    fn div() {
        const FRAME_SIZE: usize = 30;
        const MOD_SIZE: usize = FRAME_SIZE - size_of::<ModulationHead>()
            + (FRAME_SIZE - size_of::<ModulationSubseq>()) * 2;

        let device = crate::tests::create_device();

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..MOD_SIZE).map(|_| rng.random()).collect();

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::FREQ_40K,
            0,
            Segment::S0,
            transition_mode::SyncIdx.params(),
        );

        assert_eq!(op.sent, 0);

        // First frame
        {
            assert_eq!(op.required_size(&device), size_of::<ModulationHead>() + 2);
            assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

            assert_eq!(op.sent, FRAME_SIZE - size_of::<ModulationHead>());
            let mod_size = FRAME_SIZE - size_of::<ModulationHead>();

            assert_eq!(TypeTag::Modulation as u8, tx[0]);
            assert_eq!(ModulationControlFlags::BEGIN.0, tx[1]);
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
            assert_eq!(ModulationControlFlags::NONE.0, tx[1]);
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
                (ModulationControlFlags::TRANSITION | ModulationControlFlags::END).0,
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
    #[case(Err(AUTDDriverError::ModulationSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDDriverError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MIN - 1)), MOD_BUF_SIZE_MIN - 1)]
    #[case(Ok(()), MOD_BUF_SIZE_MIN)]
    #[case(Ok(()), MOD_BUF_SIZE_MAX)]
    #[case(
        Err(AUTDDriverError::ModulationSizeOutOfRange(MOD_BUF_SIZE_MAX + 1)),
        MOD_BUF_SIZE_MAX + 1
    )]
    fn out_of_range(#[case] expected: Result<(), AUTDDriverError>, #[case] size: usize) {
        let send = |n: usize| {
            use autd3_core::derive::transition_mode::Later;

            let device = crate::tests::create_device();
            let frame_size = size_of::<ModulationHead>() + device.num_transducers() * 2;
            let mut tx = vec![0x00u8; frame_size];
            let buf = Arc::new(vec![0x00; n]);
            let mut op = ModulationOp::new(
                buf.clone(),
                SamplingConfig::FREQ_40K,
                0xFFFF,
                Segment::S0,
                Later.params(),
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
    #[case(3)]
    #[case(253)]
    #[case(255)]
    fn odd_size(#[case] size: usize) -> Result<(), Box<dyn std::error::Error>> {
        let device = crate::tests::create_device();

        let mut tx = vec![0x00u8; device.num_transducers() * 2];

        let mut rng = rand::rng();

        let buf: Vec<u8> = (0..size).map(|_| rng.random()).collect();

        let mut op = ModulationOp::new(
            Arc::new(buf.clone()),
            SamplingConfig::FREQ_40K,
            0,
            Segment::S0,
            transition_mode::SyncIdx.params(),
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
