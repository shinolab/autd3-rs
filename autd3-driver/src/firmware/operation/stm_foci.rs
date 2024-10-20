use std::{mem::size_of, sync::Arc};

use crate::{
    defined::{ControlPoints, METER},
    derive::{LoopBehavior, SamplingConfig},
    error::AUTDInternalError,
    firmware::{
        fpga::{
            STMFocus, Segment, TransitionMode, FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FOCI_NUM_MAX,
            STM_BUF_SIZE_MIN, TRANSITION_MODE_NONE,
        },
        operation::{Operation, TypeTag},
    },
    geometry::Device,
};

use derive_new::new;
use zerocopy::{Immutable, IntoBytes};

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
    __pad: [u8; 4],
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

#[derive(new)]
#[new(visibility = "pub(crate)")]
pub struct FociSTMOp<const N: usize> {
    points: Arc<Vec<ControlPoints<N>>>,
    #[new(default)]
    sent: usize,
    #[new(default)]
    is_done: bool,
    config: SamplingConfig,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<const N: usize> Operation for FociSTMOp<N> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        if N == 0 || N > FOCI_STM_FOCI_NUM_MAX {
            return Err(AUTDInternalError::FociSTMNumFociOutOfRange(N));
        }

        let is_first = self.sent == 0;

        let offset = if is_first {
            size_of::<FociSTMHead>()
        } else {
            size_of::<FociSTMSubseq>()
        };

        let max_send_bytes = tx.len() - offset;
        let max_send_num = max_send_bytes / (size_of::<STMFocus>() * N);
        let send_num = (self.points.len() - self.sent).min(max_send_num);

        self.points[self.sent..]
            .iter()
            .take(send_num)
            .enumerate()
            .try_for_each(|(i, points)| {
                let intensity = points.intensity();
                let base_offset = points[0].phase_offset();
                tx[offset + i * size_of::<STMFocus>() * N..]
                    .chunks_mut(size_of::<STMFocus>())
                    .zip(points.iter())
                    .enumerate()
                    .try_for_each(|(j, (dst, p))| -> Result<_, AUTDInternalError> {
                        let lp = device.to_local(p.point());
                        dst.copy_from_slice(
                            STMFocus::create(
                                &lp,
                                if j == 0 {
                                    intensity.value()
                                } else {
                                    (p.phase_offset() - base_offset).value()
                                },
                            )?
                            .as_bytes(),
                        );
                        Ok(())
                    })
            })?;

        self.sent += send_num;
        if self.sent > FOCI_STM_BUF_SIZE_MAX {
            return Err(AUTDInternalError::FociSTMPointSizeOutOfRange(self.sent));
        }

        let mut flag = if is_first {
            FociSTMControlFlags::BEGIN
        } else {
            FociSTMControlFlags::NONE
        };
        if self.points.len() == self.sent {
            if self.sent < STM_BUF_SIZE_MIN {
                return Err(AUTDInternalError::FociSTMPointSizeOutOfRange(self.sent));
            }
            self.is_done = true;
            flag.set(FociSTMControlFlags::END, true);
            flag.set(
                FociSTMControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        if is_first {
            tx[..size_of::<FociSTMHead>()].copy_from_slice(
                FociSTMHead {
                    tag: TypeTag::FociSTM,
                    flag,
                    segment: self.segment as _,
                    transition_mode: self
                        .transition_mode
                        .map(|m| m.mode())
                        .unwrap_or(TRANSITION_MODE_NONE),
                    transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
                    send_num: send_num as _,
                    num_foci: N as u8,
                    freq_div: self.config.division(),
                    sound_speed: (device.sound_speed / METER * 64.0).round() as u16,
                    rep: self.loop_behavior.rep(),
                    __pad: [0; 4],
                }
                .as_bytes(),
            );
            Ok(size_of::<FociSTMHead>() + size_of::<STMFocus>() * send_num * N)
        } else {
            tx[..size_of::<FociSTMSubseq>()].copy_from_slice(
                FociSTMSubseq {
                    tag: TypeTag::FociSTM,
                    flag,
                    segment: self.segment as _,
                    send_num: send_num as _,
                }
                .as_bytes(),
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
        self.is_done
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::{mm, ControlPoint},
        ethercat::DcSysTime,
        firmware::fpga::{FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_UPPER_X},
        geometry::{tests::create_device, Vector3},
    };

    const NUM_TRANS_IN_UNIT: u8 = 249;

    #[test]
    fn test() {
        const FOCI_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = size_of::<FociSTMHead>() + size_of::<STMFocus>() * FOCI_STM_SIZE;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: Vec<ControlPoints<1>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    ControlPoint::new(Vector3::new(
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
            Arc::new(points.clone()),
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior { rep },
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
                    STMFocus::create(p[0].point(), p.intensity().value())
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

        let points: Vec<ControlPoints<N>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    [0; N].map(|_| {
                        ControlPoint::new(Vector3::new(
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
            Arc::new(points.clone()),
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior { rep },
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
                let base_offset = p[0].phase_offset();
                (0..N).for_each(|i| {
                    let mut buf = [0x00u8; 8];
                    buf.copy_from_slice(
                        STMFocus::create(
                            p[i].point(),
                            if i == 0 {
                                p.intensity().value()
                            } else {
                                (p[i].phase_offset() - base_offset).value()
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

        let points: Vec<ControlPoints<1>> = (0..FOCI_STM_SIZE)
            .map(|_| {
                (
                    ControlPoint::new(Vector3::new(
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(-500.0 * mm..500.0 * mm),
                        rng.gen_range(0.0 * mm..500.0 * mm),
                    )),
                    rng.gen::<u8>(),
                )
                    .into()
            })
            .collect();
        let freq_div = rng.gen_range(0x0001..=0xFFFF);
        let rep = rng.gen_range(0x0001..=0xFFFF);
        let segment = Segment::S1;

        let mut op = FociSTMOp::new(
            Arc::new(points.clone()),
            SamplingConfig::new(freq_div).unwrap(),
            LoopBehavior { rep },
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
                        STMFocus::create(p[0].point(), p.intensity().value())
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
                        STMFocus::create(p[0].point(), p.intensity().value())
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
                        STMFocus::create(p[0].point(), p.intensity().value())
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
            Arc::new(
                (0..FOCI_STM_SIZE)
                    .map(|_| ControlPoint::from(Vector3::new(x, x, x)).into())
                    .collect::<Vec<_>>(),
            ),
            SamplingConfig::FREQ_40K,
            LoopBehavior::infinite(),
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(
            op.pack(&device, &mut tx),
            Err(AUTDInternalError::FociSTMPointOutOfRange(x, x, x))
        );
    }

    #[rstest::rstest]
    #[test]
    #[case(Err(AUTDInternalError::FociSTMPointSizeOutOfRange(0)), 0)]
    #[case(Err(AUTDInternalError::FociSTMPointSizeOutOfRange(1)), 1)]
    #[case(Ok(()), 2)]
    #[case(Ok(()), FOCI_STM_BUF_SIZE_MAX)]
    #[case(Err(AUTDInternalError::FociSTMPointSizeOutOfRange(FOCI_STM_BUF_SIZE_MAX+1)), FOCI_STM_BUF_SIZE_MAX+1)]
    fn test_buffer_out_of_range(#[case] expected: Result<(), AUTDInternalError>, #[case] n: usize) {
        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut op = FociSTMOp::new(
            Arc::new(
                (0..n)
                    .map(|_| ControlPoint::new(Vector3::zeros()).into())
                    .collect::<Vec<_>>(),
            ),
            SamplingConfig::FREQ_40K,
            LoopBehavior::infinite(),
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
                Arc::new(
                    (0..2)
                        .map(|_| ControlPoints::<0>::new([]))
                        .collect::<Vec<_>>(),
                ),
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![0x00u8; size_of::<FociSTMHead>()];
            assert_eq!(
                Err(AUTDInternalError::FociSTMNumFociOutOfRange(0)),
                op.pack(&device, &mut tx).map(|_| ())
            );
        }

        {
            let mut op = FociSTMOp::new(
                Arc::new(
                    (0..2)
                        .map(|_| [0; 1].map(|_| ControlPoint::new(Vector3::zeros())).into())
                        .collect::<Vec<_>>(),
                ),
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![0x00u8; size_of::<FociSTMHead>() + 2 * size_of::<STMFocus>()];
            assert_eq!(Ok(()), op.pack(&device, &mut tx).map(|_| ()));
        }

        {
            let mut op = FociSTMOp::new(
                Arc::new(
                    (0..2)
                        .map(|_| {
                            [0; FOCI_STM_FOCI_NUM_MAX]
                                .map(|_| ControlPoint::new(Vector3::zeros()))
                                .into()
                        })
                        .collect::<Vec<_>>(),
                ),
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
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
                Arc::new(
                    (0..2)
                        .map(|_| {
                            [0; FOCI_STM_FOCI_NUM_MAX + 1]
                                .map(|_| ControlPoint::new(Vector3::zeros()))
                                .into()
                        })
                        .collect::<Vec<_>>(),
                ),
                SamplingConfig::FREQ_40K,
                LoopBehavior::infinite(),
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            let mut tx = vec![
                0x00u8;
                size_of::<FociSTMHead>()
                    + 2 * (FOCI_STM_FOCI_NUM_MAX + 1) * size_of::<STMFocus>()
            ];
            assert_eq!(
                Err(AUTDInternalError::FociSTMNumFociOutOfRange(
                    FOCI_STM_FOCI_NUM_MAX + 1
                )),
                op.pack(&device, &mut tx).map(|_| ())
            );
        }
    }
}
