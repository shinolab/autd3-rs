use std::collections::HashMap;

use crate::{
    common::{LoopBehavior, Segment},
    defined::METER,
    error::AUTDInternalError,
    fpga::{STMFocus, FOCUS_STM_BUF_SIZE_MAX},
    geometry::{Device, Geometry},
    operation::{cast, Operation, TypeTag},
};

use super::{ControlPoint, FocusSTMControlFlags};

#[repr(C)]
struct FocusSTMHead {
    tag: TypeTag,
    flag: FocusSTMControlFlags,
    send_num: u8,
    segment: u8,
    freq_div: u32,
    sound_speed: u32,
    rep: u32,
}

#[repr(C, align(2))]
struct FocusSTMSubseq {
    tag: TypeTag,
    flag: FocusSTMControlFlags,
    send_num: u8,
}

#[repr(C, align(2))]
struct FocusSTMUpdate {
    tag: TypeTag,
    segment: u8,
}

pub struct FocusSTMOp {
    remains: HashMap<usize, usize>,
    sent: HashMap<usize, usize>,
    points: Vec<ControlPoint>,
    freq_div: u32,
    loop_behavior: LoopBehavior,
    segment: Segment,
    update_segment: bool,
}

impl FocusSTMOp {
    pub fn new(
        points: Vec<ControlPoint>,
        freq_div: u32,
        loop_behavior: LoopBehavior,
        segment: Segment,
        update_segment: bool,
    ) -> Self {
        Self {
            points,
            remains: Default::default(),
            sent: Default::default(),
            freq_div,
            loop_behavior,
            segment,
            update_segment,
        }
    }
}

impl Operation for FocusSTMOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(self.remains[&device.idx()] > 0);

        let sent = self.sent[&device.idx()];

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
            let d = cast::<FocusSTMHead>(tx);
            d.tag = TypeTag::FocusSTM;
            d.flag = FocusSTMControlFlags::BEGIN;
            d.segment = self.segment as u8;
            d.send_num = send_num as u8;
            d.freq_div = self.freq_div;
            d.sound_speed = (device.sound_speed / METER * 1024.0).round() as u32;
            d.rep = self.loop_behavior.to_rep();
        } else {
            let d = cast::<FocusSTMSubseq>(tx);
            d.tag = TypeTag::FocusSTM;
            d.flag = FocusSTMControlFlags::NONE;
            d.send_num = send_num as u8;
        }

        if sent + send_num == self.points.len() {
            let d = cast::<FocusSTMSubseq>(tx);
            d.flag.set(FocusSTMControlFlags::END, true);
            d.flag
                .set(FocusSTMControlFlags::UPDATE, self.update_segment);
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

        self.sent.insert(device.idx(), sent + send_num);
        if sent == 0 {
            Ok(std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>() * send_num)
        } else {
            Ok(std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>() * send_num)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.sent[&device.idx()] == 0 {
            std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>()
        } else {
            std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if !(2..=FOCUS_STM_BUF_SIZE_MAX).contains(&self.points.len()) {
            return Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(
                self.points.len(),
            ));
        }

        self.remains = geometry
            .devices()
            .map(|device| (device.idx(), self.points.len()))
            .collect();
        self.sent = geometry.devices().map(|device| (device.idx(), 0)).collect();

        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains
            .insert(device.idx(), self.points.len() - self.sent[&device.idx()]);
    }
}

pub struct FocusSTMChangeSegmentOp {
    segment: Segment,
    remains: HashMap<usize, usize>,
}

impl FocusSTMChangeSegmentOp {
    pub fn new(segment: Segment) -> Self {
        Self {
            segment,
            remains: HashMap::new(),
        }
    }
}

impl Operation for FocusSTMChangeSegmentOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert_eq!(self.remains[&device.idx()], 1);

        let d = cast::<FocusSTMUpdate>(tx);
        d.tag = TypeTag::FocusSTMChangeSegment;
        d.segment = self.segment as u8;

        Ok(std::mem::size_of::<FocusSTMUpdate>())
    }

    fn required_size(&self, _: &Device) -> usize {
        std::mem::size_of::<FocusSTMUpdate>()
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        self.remains = geometry.devices().map(|device| (device.idx(), 1)).collect();
        Ok(())
    }

    fn remains(&self, device: &Device) -> usize {
        self.remains[&device.idx()]
    }

    fn commit(&mut self, device: &Device) {
        self.remains.insert(device.idx(), 0);
    }
}

#[cfg(test)]
mod tests {
    use std::{mem::size_of, num::NonZeroU32};

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::{float, MILLIMETER},
        fpga::{
            FOCUS_STM_FIXED_NUM_UNIT, FOCUS_STM_FIXED_NUM_WIDTH, SAMPLING_FREQ_DIV_MAX,
            SAMPLING_FREQ_DIV_MIN,
        },
        geometry::{tests::create_geometry, Vector3},
    };

    const NUM_TRANS_IN_UNIT: usize = 249;
    const NUM_DEVICE: usize = 10;

    #[test]
    fn focus_stm_op() {
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
        let loop_behavior = LoopBehavior::Infinite;
        let segment = Segment::S0;
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);

        let mut op = FocusSTMOp::new(points.clone(), freq_div, loop_behavior, segment, true);

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                size_of::<FocusSTMHead>() + size_of::<STMFocus>()
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), FOCUS_STM_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & FocusSTMControlFlags::BEGIN.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::END.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 2], FOCUS_STM_SIZE as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 3], segment as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );

            let sound_speed = (dev.sound_speed / METER * 1024.0).round() as u32;
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 8], (sound_speed & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 9],
                ((sound_speed >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 10],
                ((sound_speed >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 11],
                ((sound_speed >> 24) & 0xFF) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], 0xFF);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], 0xFF);

            tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMHead>()..]
                .chunks(size_of::<STMFocus>())
                .zip(points.iter())
                .for_each(|(d, p)| {
                    let mut f = STMFocus { buf: [0x0000; 4] };
                    f.set(p.point().x, p.point().y, p.point().z, p.intensity())
                        .unwrap();
                    assert_eq!(d[0], (f.buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((f.buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (f.buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((f.buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (f.buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((f.buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (f.buf[3] & 0xFF) as u8);
                    assert_eq!(d[7], ((f.buf[3] >> 8) & 0xFF) as u8);
                })
        });
    }

    #[test]
    fn focus_stm_op_div() {
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
        let loop_behavior = LoopBehavior::Finite(
            NonZeroU32::new(rng.gen_range(0x0000001..=0xFFFFFFFF)).unwrap_or(NonZeroU32::MIN),
        );
        let rep = loop_behavior.to_rep();
        let segment = Segment::S1;
        let mut op = FocusSTMOp::new(points.clone(), freq_div, loop_behavior, segment, false);

        assert!(op.init(&geometry).is_ok());

        // First frame
        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.required_size(dev),
                size_of::<FocusSTMHead>() + size_of::<STMFocus>()
            )
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), FOCUS_STM_SIZE));

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
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>() * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & FocusSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                ((FRAME_SIZE - size_of::<FocusSTMHead>()) / std::mem::size_of::<STMFocus>()) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 3], segment as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 4], (freq_div & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 5],
                ((freq_div >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 6],
                ((freq_div >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 7],
                ((freq_div >> 24) & 0xFF) as u8
            );

            let sound_speed = (dev.sound_speed / METER * 1024.0).round() as u32;
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 8], (sound_speed & 0xFF) as u8);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 9],
                ((sound_speed >> 8) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 10],
                ((sound_speed >> 16) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 11],
                ((sound_speed >> 24) & 0xFF) as u8
            );
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], (rep & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], ((rep >> 8) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], ((rep >> 16) & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], ((rep >> 24) & 0xFF) as u8);

            tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMHead>()..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .take((FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    let mut f = STMFocus { buf: [0x0000; 4] };
                    f.set(p.point().x, p.point().y, p.point().z, p.intensity())
                        .unwrap();
                    assert_eq!(d[0], (f.buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((f.buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (f.buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((f.buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (f.buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((f.buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (f.buf[3] & 0xFF) as u8);
                    assert_eq!(d[7], ((f.buf[3] >> 8) & 0xFF) as u8);
                })
        });

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
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & FocusSTMControlFlags::BEGIN.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                    as u8
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
                    let mut f = STMFocus { buf: [0x0000; 4] };
                    f.set(p.point().x, p.point().y, p.point().z, p.intensity())
                        .unwrap();
                    assert_eq!(d[0], (f.buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((f.buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (f.buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((f.buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (f.buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((f.buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (f.buf[3] & 0xFF) as u8);
                    assert_eq!(d[7], ((f.buf[3] >> 8) & 0xFF) as u8);
                })
        });

        // Final frame
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
            op.commit(dev);
        });

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), 0));

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & FocusSTMControlFlags::BEGIN.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::UPDATE.bits(), 0x00);
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                    as u8
            );
            tx[FRAME_SIZE * dev.idx() + size_of::<FocusSTMSubseq>()..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip(
                            (FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()
                                + (FRAME_SIZE - size_of::<FocusSTMSubseq>())
                                    / size_of::<STMFocus>(),
                        )
                        .take((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / size_of::<STMFocus>()),
                )
                .for_each(|(d, p)| {
                    let mut f = STMFocus { buf: [0x0000; 4] };
                    f.set(p.point().x, p.point().y, p.point().z, p.intensity())
                        .unwrap();
                    assert_eq!(d[0], (f.buf[0] & 0xFF) as u8);
                    assert_eq!(d[1], ((f.buf[0] >> 8) & 0xFF) as u8);
                    assert_eq!(d[2], (f.buf[1] & 0xFF) as u8);
                    assert_eq!(d[3], ((f.buf[1] >> 8) & 0xFF) as u8);
                    assert_eq!(d[4], (f.buf[2] & 0xFF) as u8);
                    assert_eq!(d[5], ((f.buf[2] >> 8) & 0xFF) as u8);
                    assert_eq!(d[6], (f.buf[3] & 0xFF) as u8);
                    assert_eq!(d[7], ((f.buf[3] >> 8) & 0xFF) as u8);
                })
        });
    }

    #[test]
    fn focus_stm_op_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let test = |n: usize| {
            let points: Vec<ControlPoint> = (0..n)
                .map(|_| ControlPoint::new(Vector3::zeros()))
                .collect();
            let mut op = FocusSTMOp::new(
                points,
                SAMPLING_FREQ_DIV_MIN,
                LoopBehavior::Infinite,
                Segment::S0,
                true,
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
    fn focus_stm_op_point_out_of_range() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = 16 + 8 * FOCUS_STM_SIZE;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let x = FOCUS_STM_FIXED_NUM_UNIT * (1 << (FOCUS_STM_FIXED_NUM_WIDTH - 1)) as float;
        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| ControlPoint::new(Vector3::new(x, x, x)).with_intensity(0))
            .collect();
        let freq_div: u32 = SAMPLING_FREQ_DIV_MIN;

        let mut op = FocusSTMOp::new(
            points.clone(),
            freq_div,
            LoopBehavior::Infinite,
            Segment::S0,
            true,
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Err(AUTDInternalError::FocusSTMPointOutOfRange(x))
            );
        });
    }
}
