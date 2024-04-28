use crate::{
    defined::METER,
    error::AUTDInternalError,
    firmware::{
        fpga::{
            LoopBehavior, STMFocus, Segment, TransitionMode, FOCUS_STM_BUF_SIZE_MAX,
            STM_BUF_SIZE_MIN,
        },
        operation::{cast, Operation, Remains, TypeTag},
    },
    geometry::{Device, Geometry},
};

use super::{ControlPoint, FocusSTMControlFlags};

#[repr(C, align(2))]
struct FocusSTMHead {
    tag: TypeTag,
    flag: FocusSTMControlFlags,
    send_num: u8,
    transition_mode: u8,
    freq_div: u32,
    sound_speed: u32,
    rep: u32,
    transition_value: u64,
}

#[repr(C, align(2))]
struct FocusSTMSubseq {
    tag: TypeTag,
    flag: FocusSTMControlFlags,
    send_num: u8,
}

pub struct FocusSTMOp {
    remains: Remains,
    points: Vec<ControlPoint>,
    freq_div: u32,
    loop_behavior: LoopBehavior,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl FocusSTMOp {
    pub fn new(
        points: Vec<ControlPoint>,
        freq_div: u32,
        loop_behavior: LoopBehavior,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            points,
            remains: Default::default(),
            freq_div,
            loop_behavior,
            segment,
            transition_mode,
        }
    }
}

impl Operation for FocusSTMOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = self.points.len() - self.remains[device];

        let offset = if sent == 0 {
            std::mem::size_of::<FocusSTMHead>()
        } else {
            std::mem::size_of::<FocusSTMSubseq>()
        };

        let send_bytes =
            ((self.points.len() - sent) * std::mem::size_of::<STMFocus>()).min(tx.len() - offset);
        let send_num = send_bytes / std::mem::size_of::<STMFocus>();
        assert!(send_num > 0);

        if sent == 0 {
            *cast::<FocusSTMHead>(tx) = FocusSTMHead {
                tag: TypeTag::FocusSTM,
                flag: FocusSTMControlFlags::BEGIN
                    | if self.segment == Segment::S1 {
                        FocusSTMControlFlags::SEGMENT
                    } else {
                        FocusSTMControlFlags::NONE
                    },
                transition_mode: self.transition_mode.unwrap_or_default().mode(),
                transition_value: self.transition_mode.unwrap_or_default().value(),
                send_num: send_num as u8,
                freq_div: self.freq_div,
                sound_speed: (device.sound_speed / METER
                    * 1024.0
                    * crate::firmware::fpga::FREQ_40K as f64
                    / crate::firmware::fpga::ultrasound_freq() as f64)
                    .round() as u32,
                rep: self.loop_behavior.rep,
            };
        } else {
            *cast::<FocusSTMSubseq>(tx) = FocusSTMSubseq {
                tag: TypeTag::FocusSTM,
                flag: FocusSTMControlFlags::NONE,
                send_num: send_num as u8,
            };
        }

        if sent + send_num == self.points.len() {
            let d = cast::<FocusSTMSubseq>(tx);
            d.flag.set(FocusSTMControlFlags::END, true);
            d.flag.set(
                FocusSTMControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        unsafe {
            std::slice::from_raw_parts_mut(tx[offset..].as_mut_ptr() as *mut STMFocus, send_num)
                .iter_mut()
                .zip(self.points.iter().skip(sent).take(send_num))
                .try_for_each(|(d, p)| {
                    let lp = device.to_local(p.point());
                    d.set(lp.x, lp.y, lp.z, p.intensity())
                })?
        }

        self.remains.send(device, send_num);
        if sent == 0 {
            Ok(std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>() * send_num)
        } else {
            Ok(std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>() * send_num)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.remains[device] == self.points.len() {
            std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>()
        } else {
            std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if !(STM_BUF_SIZE_MIN..=FOCUS_STM_BUF_SIZE_MAX).contains(&self.points.len()) {
            return Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(
                self.points.len(),
            ));
        }

        self.remains.init(geometry, self.points.len());

        Ok(())
    }

    fn is_done(&self, device: &Device) -> bool {
        self.remains.is_done(device)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::MILLIMETER,
        ethercat::DcSysTime,
        firmware::{
            fpga::{
                FOCUS_STM_FIXED_NUM_UNIT, FOCUS_STM_FIXED_NUM_UPPER_X, SAMPLING_FREQ_DIV_MAX,
                SAMPLING_FREQ_DIV_MIN,
            },
            operation::tests::parse_tx_as,
        },
        geometry::{tests::create_geometry, Vector3},
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn test() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize =
            size_of::<FocusSTMHead>() + size_of::<STMFocus>() * FOCUS_STM_SIZE;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| {
                ControlPoint::new(Vector3::new(
                    rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                    rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                    rng.gen_range(0.0 * MILLIMETER..500.0 * MILLIMETER),
                ))
                .with_intensity(rng.gen::<u8>())
            })
            .collect();
        let loop_behavior = LoopBehavior::infinite();
        let rep = loop_behavior.rep;
        let segment = Segment::S0;
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(time::macros::datetime!(2000-01-01 0:00 UTC)).unwrap()
                + std::time::Duration::from_nanos(transition_value),
        );

        let mut op = FocusSTMOp::new(
            points.clone(),
            freq_div,
            loop_behavior,
            segment,
            Some(transition_mode),
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                size_of::<FocusSTMHead>() + size_of::<STMFocus>()
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], FOCUS_STM_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(FRAME_SIZE)
            );
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains[dev], 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(TypeTag::FocusSTM as u8, tx[dev.idx() * FRAME_SIZE]);
            assert_eq!(
                (FocusSTMControlFlags::BEGIN
                    | FocusSTMControlFlags::END
                    | FocusSTMControlFlags::TRANSITION)
                    .bits(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, flag)]
            );
            assert_eq!(
                FOCUS_STM_SIZE as u8,
                tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, send_num)]
            );
            assert_eq!(
                freq_div,
                parse_tx_as::<u32>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, freq_div)..]
                )
            );
            let sound_speed = (dev.sound_speed / METER * 1024.0).round() as u32;
            assert_eq!(
                sound_speed,
                parse_tx_as::<u32>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, sound_speed)..]
                )
            );
            assert_eq!(
                rep,
                parse_tx_as::<u32>(&tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, rep)..])
            );
            assert_eq!(
                transition_mode.mode(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, transition_mode)]
            );
            assert_eq!(
                transition_value,
                parse_tx_as::<u64>(
                    &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, transition_value)..]
                )
            );
            tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMHead>()..]
                .chunks(size_of::<STMFocus>())
                .zip(points.iter())
                .for_each(|(d, p)| {
                    let mut buf = [0x0000u16; 4];
                    unsafe {
                        let _ = (*(&mut buf as *mut _ as *mut STMFocus)).set(
                            p.point().x,
                            p.point().y,
                            p.point().z,
                            p.intensity(),
                        );
                    }
                    assert_eq!(d[0], (buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (buf[3] & 0xFF) as u8);
                    assert_eq!(d[7] & 0x3F, ((buf[3] >> 8) & 0xFF) as u8);
                })
        });
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 32;
        const FOCUS_STM_SIZE: usize = (FRAME_SIZE - size_of::<FocusSTMHead>())
            / size_of::<STMFocus>()
            + (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / size_of::<STMFocus>() * 2;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| {
                ControlPoint::new(Vector3::new(
                    rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                    rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                    rng.gen_range(0.0 * MILLIMETER..500.0 * MILLIMETER),
                ))
                .with_intensity(rng.gen::<u8>())
            })
            .collect();
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let loop_behavior = LoopBehavior::finite(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap();
        let rep = loop_behavior.rep;
        let segment = Segment::S1;
        let mut op = FocusSTMOp::new(points.clone(), freq_div, loop_behavior, segment, None);

        assert!(op.init(&geometry).is_ok());

        // First frame
        {
            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.required_size(dev),
                    size_of::<FocusSTMHead>() + size_of::<STMFocus>()
                )
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], FOCUS_STM_SIZE));

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.pack(
                        dev,
                        &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                    ),
                    Ok(size_of::<FocusSTMHead>()
                        + (FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()
                            * size_of::<STMFocus>())
                );
            });

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.remains[dev],
                    (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
                        * 2
                )
            });

            geometry.devices().for_each(|dev| {
                assert_eq!(TypeTag::FocusSTM as u8, tx[dev.idx() * FRAME_SIZE]);
                assert_eq!(
                    (FocusSTMControlFlags::BEGIN | FocusSTMControlFlags::SEGMENT).bits(),
                    tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, flag)]
                );
                assert_eq!(
                    ((FRAME_SIZE - size_of::<FocusSTMHead>()) / std::mem::size_of::<STMFocus>())
                        as u8,
                    tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, send_num)],
                );
                assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
                assert_eq!(
                    freq_div,
                    parse_tx_as::<u32>(
                        &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, freq_div)..]
                    )
                );
                let sound_speed = (dev.sound_speed / METER * 1024.0).round() as u32;
                assert_eq!(
                    sound_speed,
                    parse_tx_as::<u32>(
                        &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, sound_speed)..]
                    )
                );
                assert_eq!(
                    rep,
                    parse_tx_as::<u32>(
                        &tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, rep)..]
                    )
                );

                tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMHead>()..FRAME_SIZE * (dev.idx() + 1)]
                    .chunks(size_of::<STMFocus>())
                    .zip(
                        points
                            .iter()
                            .take((FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()),
                    )
                    .for_each(|(d, p)| {
                        let mut buf = [0x0000u16; 4];
                        unsafe {
                            let _ = (*(&mut buf as *mut _ as *mut STMFocus)).set(
                                p.point().x,
                                p.point().y,
                                p.point().z,
                                p.intensity(),
                            );
                        }
                        assert_eq!(d[0], (buf[0] & 0xFF) as u8);
                        assert_eq!(d[1], ((buf[0] >> 8) & 0xFF) as u8);
                        assert_eq!(d[2], (buf[1] & 0xFF) as u8);
                        assert_eq!(d[3], ((buf[1] >> 8) & 0xFF) as u8);
                        assert_eq!(d[4], (buf[2] & 0xFF) as u8);
                        assert_eq!(d[5], ((buf[2] >> 8) & 0xFF) as u8);
                        assert_eq!(d[6], (buf[3] & 0xFF) as u8);
                        assert_eq!(d[7] & 0x3F, ((buf[3] >> 8) & 0xFF) as u8);
                    })
            });
        }

        // Second frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(size_of::<FocusSTMSubseq>()
                    + (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
                        * std::mem::size_of::<STMFocus>())
            );
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains[dev],
                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(TypeTag::FocusSTM as u8, tx[dev.idx() * FRAME_SIZE]);
            assert_eq!(
                FocusSTMControlFlags::NONE.bits(),
                tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, flag)]
            );
            assert_eq!(
                ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                    as u8,
                tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, send_num)],
            );
            tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMSubseq>()..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip((FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>())
                        .take((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    let mut buf = [0x0000u16; 4];
                    unsafe {
                        let _ = (*(&mut buf as *mut _ as *mut STMFocus)).set(
                            p.point().x,
                            p.point().y,
                            p.point().z,
                            p.intensity(),
                        );
                    }
                    assert_eq!(d[0], (buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (buf[3] & 0xFF) as u8);
                    assert_eq!(d[7] & 0x3F, ((buf[3] >> 8) & 0xFF) as u8);
                })
        });

        // Final frame
        {
            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.required_size(dev),
                    size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
                )
            });

            geometry.devices().for_each(|dev| {
                assert_eq!(
                    op.pack(
                        dev,
                        &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                    ),
                    Ok(size_of::<FocusSTMSubseq>()
                        + (FRAME_SIZE - size_of::<FocusSTMSubseq>())
                            / std::mem::size_of::<STMFocus>()
                            * std::mem::size_of::<STMFocus>())
                );
            });

            geometry
                .devices()
                .for_each(|dev| assert_eq!(op.remains[dev], 0));

            geometry.devices().for_each(|dev| {
                assert_eq!(TypeTag::FocusSTM as u8, tx[dev.idx() * FRAME_SIZE]);
                assert_eq!(
                    FocusSTMControlFlags::END.bits(),
                    tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, flag)]
                );
                assert_eq!(
                    ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                        as u8,
                    tx[dev.idx() * FRAME_SIZE + offset_of!(FocusSTMHead, send_num)],
                );
                tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMSubseq>()
                    ..FRAME_SIZE * (dev.idx() + 1)]
                    .chunks(size_of::<STMFocus>())
                    .zip(
                        points
                            .iter()
                            .skip(
                                (FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()
                                    + (FRAME_SIZE - size_of::<FocusSTMSubseq>())
                                        / size_of::<STMFocus>(),
                            )
                            .take(
                                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / size_of::<STMFocus>(),
                            ),
                    )
                    .for_each(|(d, p)| {
                        let mut buf = [0x0000u16; 4];
                        unsafe {
                            let _ = (*(&mut buf as *mut _ as *mut STMFocus)).set(
                                p.point().x,
                                p.point().y,
                                p.point().z,
                                p.intensity(),
                            );
                        }
                        assert_eq!(d[0], (buf[0] & 0xFF) as u8);
                        assert_eq!(d[1], ((buf[0] >> 8) & 0xFF) as u8);
                        assert_eq!(d[2], (buf[1] & 0xFF) as u8);
                        assert_eq!(d[3], ((buf[1] >> 8) & 0xFF) as u8);
                        assert_eq!(d[4], (buf[2] & 0xFF) as u8);
                        assert_eq!(d[5], ((buf[2] >> 8) & 0xFF) as u8);
                        assert_eq!(d[6], (buf[3] & 0xFF) as u8);
                        assert_eq!(d[7] & 0x3F, ((buf[3] >> 8) & 0xFF) as u8);
                    })
            });
        }
    }

    #[test]
    fn test_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let test = |n: usize| {
            let points: Vec<ControlPoint> = (0..n)
                .map(|_| ControlPoint::new(Vector3::zeros()))
                .collect();
            let mut op = FocusSTMOp::new(
                points,
                SAMPLING_FREQ_DIV_MIN,
                LoopBehavior::infinite(),
                Segment::S0,
                Some(TransitionMode::SyncIdx),
            );
            op.init(&geometry)
        };

        assert_eq!(
            test(1),
            Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(1))
        );
        assert_eq!(test(2), Ok(()));
        assert_eq!(test(FOCUS_STM_BUF_SIZE_MAX), Ok(()));
        assert_eq!(
            test(FOCUS_STM_BUF_SIZE_MAX + 1),
            Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(
                FOCUS_STM_BUF_SIZE_MAX + 1
            ))
        );
    }

    #[test]
    fn test_point_out_of_range() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = 16 + 8 * FOCUS_STM_SIZE;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let x = FOCUS_STM_FIXED_NUM_UNIT * (FOCUS_STM_FIXED_NUM_UPPER_X as f64 + 1.);
        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| ControlPoint::new(Vector3::new(x, x, x)).with_intensity(0))
            .collect();
        let freq_div: u32 = SAMPLING_FREQ_DIV_MIN;

        let mut op = FocusSTMOp::new(
            points.clone(),
            freq_div,
            LoopBehavior::infinite(),
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Err(AUTDInternalError::FocusSTMPointOutOfRange(x, x, x))
            );
        });
    }
}
