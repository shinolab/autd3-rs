use std::mem::size_of;

use crate::{
    defined::METER,
    error::AUTDDriverError,
    firmware::{
        fpga::{
            LoopBehavior, STMFocus, SamplingConfig, Segment, TransitionMode, FOCI_STM_BUF_SIZE_MAX,
            FOCI_STM_FOCI_NUM_MAX, STM_BUF_SIZE_MIN, TRANSITION_MODE_NONE,
        },
        operation::{write_to_tx, Operation, TypeTag},
    },
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

use super::ControlPoints;

#[derive(Clone, Copy, PartialEq, Debug, IntoBytes, Immutable)]
#[repr(C)]
pub struct FociSTMControlFlags(u8);

bitflags::bitflags! {
    impl FociSTMControlFlags : u8 {
        const NONE       = 0;
        const BEGIN      = 1 << 0;
        const END        = 1 << 1;
        const TRANSITION = 1 << 2;
    }
}

#[repr(C, align(2))]
#[derive(PartialEq, Debug, IntoBytes, Immutable)]
struct FociSTMHead {
    tag: TypeTag,
    flag: FociSTMControlFlags,
    send_num: u8,
    segment: u8,
    transition_mode: u8,
    num_foci: u8,
    sound_speed: u16,
    freq_div: u16,
    rep: u16,
    __: [u8; 4],
    transition_value: u64,
}

#[repr(C, align(2))]
#[derive(IntoBytes, Immutable)]
struct FociSTMSubseq {
    tag: TypeTag,
    flag: FociSTMControlFlags,
    send_num: u8,
    segment: u8,
}

/// A trait to generate a [`ControlPoints`] for  [`FociSTM`].
///
/// [`FociSTM`]: crate::datagram::FociSTM
pub trait FociSTMContext<const N: usize>: Send + Sync {
    /// Returns the next [`ControlPoints`].
    fn next(&mut self) -> ControlPoints<N>;
}

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct FociSTMOp<const N: usize, Context: FociSTMContext<N>> {
    context: Context,
    size: usize,
    #[new(default)]
    sent: usize,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize, Context: FociSTMContext<N>> Operation for FociSTMOp<N, Context> {
    type Error = AUTDDriverError;

    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDDriverError> {
        if N == 0 || N > FOCI_STM_FOCI_NUM_MAX {
            return Err(AUTDDriverError::FociSTMNumFociOutOfRange(N));
        }
        if !(STM_BUF_SIZE_MIN..=FOCI_STM_BUF_SIZE_MAX).contains(&self.size) {
            return Err(AUTDDriverError::FociSTMPointSizeOutOfRange(self.size));
        }

        let is_first = self.sent == 0;

        let send_num = {
            let offset = if is_first {
                size_of::<FociSTMHead>()
            } else {
                size_of::<FociSTMSubseq>()
            };

            let max_send_bytes = tx.len() - offset;
            let max_send_num = max_send_bytes / (size_of::<STMFocus>() * N);
            let send_num = (self.size - self.sent).min(max_send_num);

            let mut idx = offset;
            (0..send_num).try_for_each(|_| {
                let p = self.context.next();
                let p = p.transform(device.inv());
                write_to_tx(
                    &mut tx[idx..],
                    STMFocus::create(&p[0].point, p.intensity.0)?,
                );
                idx += size_of::<STMFocus>();
                (1..N).try_for_each(|i| {
                    write_to_tx(
                        &mut tx[idx..],
                        STMFocus::create(&p[i].point, (p[i].phase_offset - p[0].phase_offset).0)?,
                    );
                    idx += size_of::<STMFocus>();
                    Result::<_, AUTDDriverError>::Ok(())
                })
            })?;

            send_num
        };

        self.sent += send_num;

        let flag = if self.size == self.sent {
            FociSTMControlFlags::END
                | if self.transition_mode.is_some() {
                    FociSTMControlFlags::TRANSITION
                } else {
                    FociSTMControlFlags::NONE
                }
        } else {
            FociSTMControlFlags::NONE
        };
        if is_first {
            write_to_tx(
                tx,
                FociSTMHead {
                    tag: TypeTag::FociSTM,
                    flag: flag | FociSTMControlFlags::BEGIN,
                    segment: self.segment as _,
                    transition_mode: self
                        .transition_mode
                        .map(|m| m.mode())
                        .unwrap_or(TRANSITION_MODE_NONE),
                    transition_value: self.transition_mode.map(TransitionMode::value).unwrap_or(0),
                    send_num: send_num as _,
                    num_foci: N as u8,
                    freq_div: self.config.division.get(),
                    sound_speed: (device.sound_speed / METER * 64.0).round() as u16,
                    rep: self.loop_behavior.rep(),
                    __: [0; 4],
                },
            );
            Ok(size_of::<FociSTMHead>() + size_of::<STMFocus>() * send_num * N)
        } else {
            write_to_tx(
                tx,
                FociSTMSubseq {
                    tag: TypeTag::FociSTM,
                    flag,
                    segment: self.segment as _,
                    send_num: send_num as _,
                },
            );
            Ok(size_of::<FociSTMSubseq>() + size_of::<STMFocus>() * send_num * N)
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.sent == 0 {
            size_of::<FociSTMHead>() + size_of::<STMFocus>() * N
        } else {
            size_of::<FociSTMSubseq>() + size_of::<STMFocus>() * N
        }
    }

    fn is_done(&self) -> bool {
        self.size == self.sent
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        mem::{offset_of, size_of},
        num::NonZeroU16,
    };

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::mm,
        ethercat::DcSysTime,
        firmware::{
            fpga::{FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_UPPER_X},
            operation::{tests::create_device, ControlPoint},
        },
        geometry::Point3,
    };

    const NUM_TRANS_IN_UNIT: u8 = 249;

    struct TestContext<const N: usize> {
        points: VecDeque<ControlPoints<N>>,
    }

    impl<const N: usize> FociSTMContext<N> for TestContext<N> {
        fn next(&mut self) -> ControlPoints<N> {
            self.points.pop_front().unwrap()
        }
    }

    #[test]
    fn test() {
        const FOCI_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = size_of::<FociSTMHead>() + size_of::<STMFocus>() * FOCI_STM_SIZE;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: VecDeque<ControlPoints<1>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    ControlPoint::from(Point3::new(
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(0.0 * mm..500.0 * mm),
                    )),
                    rng.gen::<u8>(),
                )
                    .into()
            })
            .collect();
        let rep = 0xFFFF;
        let segment = Segment::S0;
        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(
                time::macros::datetime!(2000-01-01 0:00 UTC)
                    + std::time::Duration::from_nanos(transition_value),
            )
            .unwrap(),
        );

        let mut op = FociSTMOp::new(
            TestContext {
                points: points.clone(),
            },
            FOCI_STM_SIZE,
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior::Infinite,
            segment,
            Some(transition_mode),
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<FociSTMHead>() + size_of::<STMFocus>()
        );

        assert_eq!(op.sent, 0);

        assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

        assert_eq!(op.sent, FOCI_STM_SIZE);

        assert_eq!(TypeTag::FociSTM as u8, tx[0]);
        assert_eq!(
            (FociSTMControlFlags::BEGIN
                | FociSTMControlFlags::END
                | FociSTMControlFlags::TRANSITION)
                .bits(),
            tx[1]
        );
        assert_eq!(
            ((FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>()) as u8,
            tx[2]
        );
        assert_eq!(segment as u8, tx[3]);
        assert_eq!(transition_mode.mode(), tx[4]);
        assert_eq!(1, tx[5]);
        let sound_speed = (device.sound_speed / METER * 64.0).round() as u16;
        assert_eq!(sound_speed as u8, tx[6]);
        assert_eq!((sound_speed >> 8) as u8, tx[7]);
        assert_eq!(freq_div as u8, tx[8]);
        assert_eq!((freq_div >> 8) as u8, tx[9]);
        assert_eq!(rep as u8, tx[10]);
        assert_eq!((rep >> 8) as u8, tx[11]);
        assert_eq!(transition_value as u8, tx[16]);
        assert_eq!((transition_value >> 8) as u8, tx[17]);
        assert_eq!((transition_value >> 16) as u8, tx[18]);
        assert_eq!((transition_value >> 24) as u8, tx[19]);
        assert_eq!((transition_value >> 32) as u8, tx[20]);
        assert_eq!((transition_value >> 40) as u8, tx[21]);
        assert_eq!((transition_value >> 48) as u8, tx[22]);
        assert_eq!((transition_value >> 56) as u8, tx[23]);
        tx[size_of::<FociSTMHead>()..]
            .chunks(size_of::<STMFocus>())
            .zip(points.iter())
            .for_each(|(d, p)| {
                assert_eq!(
                    d,
                    STMFocus::create(&p[0].point, p.intensity.0)
                        .unwrap()
                        .as_bytes()
                );
            });
    }

    #[test]
    fn test_foci() {
        const FOCI_STM_SIZE: usize = 10;
        const N: usize = 8;
        const FRAME_SIZE: usize =
            size_of::<FociSTMHead>() + size_of::<STMFocus>() * FOCI_STM_SIZE * N;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: VecDeque<ControlPoints<N>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    [0; N].map(|_| {
                        ControlPoint::from(Point3::new(
                            rng.gen_range(-500.0 * mm..500.0 * mm),
                            rng.gen_range(-500.0 * mm..500.0 * mm),
                            rng.gen_range(0.0 * mm..500.0 * mm),
                        ))
                    }),
                    rng.gen::<u8>(),
                )
                    .into()
            })
            .collect();
        let rep = 0xFFFF;
        let segment = Segment::S0;
        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(
                time::macros::datetime!(2000-01-01 0:00 UTC)
                    + std::time::Duration::from_nanos(transition_value),
            )
            .unwrap(),
        );

        let mut op = FociSTMOp::new(
            TestContext {
                points: points.clone(),
            },
            FOCI_STM_SIZE,
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior::Infinite,
            segment,
            Some(transition_mode),
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<FociSTMHead>() + size_of::<STMFocus>() * N
        );

        assert_eq!(op.sent, 0);

        assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

        assert_eq!(op.sent, FOCI_STM_SIZE);

        assert_eq!(TypeTag::FociSTM as u8, tx[0]);
        assert_eq!(
            (FociSTMControlFlags::BEGIN
                | FociSTMControlFlags::END
                | FociSTMControlFlags::TRANSITION)
                .bits(),
            tx[1]
        );
        assert_eq!(FOCI_STM_SIZE as u8, tx[2]);
        assert_eq!(segment as u8, tx[3]);
        assert_eq!(transition_mode.mode(), tx[4]);
        assert_eq!(N as u8, tx[5]);
        let sound_speed = (device.sound_speed / METER * 64.0).round() as u16;
        assert_eq!(sound_speed as u8, tx[6]);
        assert_eq!((sound_speed >> 8) as u8, tx[7]);
        assert_eq!(freq_div as u8, tx[8]);
        assert_eq!((freq_div >> 8) as u8, tx[9]);
        assert_eq!(rep as u8, tx[10]);
        assert_eq!((rep >> 8) as u8, tx[11]);
        assert_eq!(transition_value as u8, tx[16]);
        assert_eq!((transition_value >> 8) as u8, tx[17]);
        assert_eq!((transition_value >> 16) as u8, tx[18]);
        assert_eq!((transition_value >> 24) as u8, tx[19]);
        assert_eq!((transition_value >> 32) as u8, tx[20]);
        assert_eq!((transition_value >> 40) as u8, tx[21]);
        assert_eq!((transition_value >> 48) as u8, tx[22]);
        assert_eq!((transition_value >> 56) as u8, tx[23]);
        tx[size_of::<FociSTMHead>()..]
            .chunks(size_of::<STMFocus>() * N)
            .zip(points.iter())
            .for_each(|(d, p)| {
                let base_offset = p[0].phase_offset;
                (0..N).for_each(|i| {
                    let mut buf = [0x00u8; 8];
                    buf.copy_from_slice(
                        STMFocus::create(
                            &p[i].point,
                            if i == 0 {
                                p.intensity.0
                            } else {
                                (p[i].phase_offset - base_offset).0
                            },
                        )
                        .unwrap()
                        .as_bytes(),
                    );
                    assert_eq!(
                        d[i * size_of::<STMFocus>()..i * size_of::<STMFocus>() + 8],
                        buf
                    );
                });
            });
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 32;
        const FOCI_STM_SIZE: usize = 7;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: VecDeque<ControlPoints<1>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    ControlPoint::from(Point3::new(
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(0.0 * mm..500.0 * mm),
                    )),
                    rng.gen::<u8>(),
                )
                    .into()
            })
            .collect();
        let freq_div = rng.gen_range(0x0001..0xFFFF);
        let rep = rng.gen_range(0x0001..=0xFFFF);
        let segment = Segment::S1;

        let mut op = FociSTMOp::new(
            TestContext {
                points: points.clone(),
            },
            FOCI_STM_SIZE,
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior::Finite(NonZeroU16::new(rep + 1).unwrap()),
            segment,
            None,
        );

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FociSTMHead>() + size_of::<STMFocus>()
            );

            assert_eq!(op.sent, 0);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<FociSTMHead>()
                    + (FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>()
                        * size_of::<STMFocus>())
            );

            assert_eq!(op.sent, 1);

            assert_eq!(TypeTag::FociSTM as u8, tx[0]);
            assert_eq!(FociSTMControlFlags::BEGIN.bits(), tx[1]);
            assert_eq!(
                ((FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>()) as u8,
                tx[2]
            );
            assert_eq!(segment as u8, tx[3]);
            assert_eq!(1, tx[5]);
            let sound_speed = (device.sound_speed / METER * 64.0).round() as u16;
            assert_eq!(sound_speed as u8, tx[6]);
            assert_eq!((sound_speed >> 8) as u8, tx[7]);
            assert_eq!(freq_div as u8, tx[8]);
            assert_eq!((freq_div >> 8) as u8, tx[9]);
            assert_eq!(rep as u8, tx[10]);
            assert_eq!((rep >> 8) as u8, tx[11]);

            tx[size_of::<FociSTMHead>()..]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .take((FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    assert_eq!(
                        d,
                        STMFocus::create(&p[0].point, p.intensity.0)
                            .unwrap()
                            .as_bytes()
                    );
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FociSTMSubseq>() + size_of::<STMFocus>()
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<FociSTMSubseq>()
                    + (FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()
                        * size_of::<STMFocus>())
            );

            assert_eq!(op.sent, 4);

            assert_eq!(TypeTag::FociSTM as u8, tx[0]);
            assert_eq!(1, tx[offset_of!(FociSTMHead, segment)]);
            assert_eq!(
                ((FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()) as u8,
                tx[offset_of!(FociSTMHead, send_num)],
            );
            tx[size_of::<FociSTMSubseq>()..]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip((FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>())
                        .take((FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    assert_eq!(
                        d,
                        STMFocus::create(&p[0].point, p.intensity.0)
                            .unwrap()
                            .as_bytes()
                    );
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FociSTMSubseq>() + size_of::<STMFocus>()
            );

            assert_eq!(
                op.pack(&device, &mut tx[0..(&device.idx() + 1) * FRAME_SIZE]),
                Ok(size_of::<FociSTMSubseq>()
                    + (FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()
                        * size_of::<STMFocus>())
            );

            assert_eq!(op.sent, FOCI_STM_SIZE);

            assert_eq!(TypeTag::FociSTM as u8, tx[0]);
            assert_eq!(
                FociSTMControlFlags::END.bits(),
                tx[offset_of!(FociSTMHead, flag)]
            );
            assert_eq!(
                ((FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()) as u8,
                tx[offset_of!(FociSTMHead, send_num)],
            );
            tx[size_of::<FociSTMSubseq>()..]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip(
                            (FRAME_SIZE - size_of::<FociSTMHead>()) / size_of::<STMFocus>()
                                + (FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>(),
                        )
                        .take((FRAME_SIZE - size_of::<FociSTMSubseq>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    assert_eq!(
                        d,
                        STMFocus::create(&p[0].point, p.intensity.0)
                            .unwrap()
                            .as_bytes()
                    );
                })
        }
    }

    #[test]
    fn test_point_out_of_range() {
        const FOCI_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = 16 + 8 * FOCI_STM_SIZE;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let x = FOCI_STM_FIXED_NUM_UNIT * (FOCI_STM_FIXED_NUM_UPPER_X as f32 + 1.);

        let mut op = FociSTMOp::new(
            TestContext {
                points: (0..FOCI_STM_SIZE)
                    .map(|_| ControlPoint::from(Point3::new(x, x, x)).into())
                    .collect::<VecDeque<_>>(),
            },
            FOCI_STM_SIZE,
            SamplingConfig::FREQ_MAX,
            LoopBehavior::Infinite,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(
            op.pack(&device, &mut tx),
            Err(AUTDDriverError::FociSTMPointOutOfRange(x, x, x))
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDDriverError::FociSTMPointSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDDriverError::FociSTMPointSizeOutOfRange(1)), 1)]
    #[case(Ok(()), 2)]
    #[case(Ok(()), FOCI_STM_BUF_SIZE_MAX)]
    #[case(Err(AUTDDriverError::FociSTMPointSizeOutOfRange(FOCI_STM_BUF_SIZE_MAX+1)), FOCI_STM_BUF_SIZE_MAX+1)]
    fn test_buffer_out_of_range(#[case] expected: Result<(), AUTDDriverError>, #[case] n: usize) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut op = FociSTMOp::new(
            TestContext {
                points: (0..n)
                    .map(|_| ControlPoint::from(Point3::origin()).into())
                    .collect::<VecDeque<_>>(),
            },
            n,
            SamplingConfig::FREQ_MAX,
            LoopBehavior::Infinite,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        let mut tx = vec![0x00u8; size_of::<FociSTMHead>() + n * size_of::<STMFocus>()];

        assert_eq!(op.pack(&device, &mut tx).map(|_| ()), expected);
    }

    #[test]
    fn test_foci_out_of_range() {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        {
            let mut op = FociSTMOp::new(
                TestContext {
                    points: (0..2)
                        .map(|_| ControlPoints::<0>::default())
                        .collect::<VecDeque<_>>(),
                },
                2,
                SamplingConfig::FREQ_MAX,
                LoopBehavior::Infinite,
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![0x00u8; size_of::<FociSTMHead>()];
            assert_eq!(
                Err(AUTDDriverError::FociSTMNumFociOutOfRange(0)),
                op.pack(&device, &mut tx).map(|_| ())
            );
        }

        {
            let mut op = FociSTMOp::new(
                TestContext {
                    points: (0..2)
                        .map(|_| [0; 1].map(|_| ControlPoint::from(Point3::origin())).into())
                        .collect::<VecDeque<_>>(),
                },
                2,
                SamplingConfig::FREQ_MAX,
                LoopBehavior::Infinite,
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![0x00u8; size_of::<FociSTMHead>() + 2 * size_of::<STMFocus>()];
            assert_eq!(Ok(()), op.pack(&device, &mut tx).map(|_| ()));
        }

        {
            let mut op = FociSTMOp::new(
                TestContext {
                    points: (0..2)
                        .map(|_| {
                            [0; FOCI_STM_FOCI_NUM_MAX]
                                .map(|_| ControlPoint::from(Point3::origin()))
                                .into()
                        })
                        .collect::<VecDeque<_>>(),
                },
                2,
                SamplingConfig::FREQ_MAX,
                LoopBehavior::Infinite,
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![
                0x00u8;
                size_of::<FociSTMHead>()
                    + 2 * FOCI_STM_FOCI_NUM_MAX * size_of::<STMFocus>()
            ];
            assert_eq!(Ok(()), op.pack(&device, &mut tx).map(|_| ()));
        }

        {
            let mut op = FociSTMOp::new(
                TestContext {
                    points: (0..2)
                        .map(|_| {
                            [0; FOCI_STM_FOCI_NUM_MAX + 1]
                                .map(|_| ControlPoint::from(Point3::origin()))
                                .into()
                        })
                        .collect::<VecDeque<_>>(),
                },
                2,
                SamplingConfig::FREQ_MAX,
                LoopBehavior::Infinite,
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![
                0x00u8;
                size_of::<FociSTMHead>()
                    + 2 * (FOCI_STM_FOCI_NUM_MAX + 1) * size_of::<STMFocus>()
            ];
            assert_eq!(
                Err(AUTDDriverError::FociSTMNumFociOutOfRange(
                    FOCI_STM_FOCI_NUM_MAX + 1
                )),
                op.pack(&device, &mut tx).map(|_| ())
            );
        }
    }
}
