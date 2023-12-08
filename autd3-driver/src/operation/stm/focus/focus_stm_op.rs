/*
 * File: focus_stm_op.rs
 * Project: focus
 * Created Date: 06/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 27/11/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use std::collections::HashMap;

use crate::{
    defined::METER,
    error::AUTDInternalError,
    fpga::{STMFocus, FOCUS_STM_BUF_SIZE_MAX},
    geometry::{Device, Geometry},
    operation::{Operation, TypeTag},
};

use super::{ControlPoint, FocusSTMControlFlags};

pub struct FocusSTMOp {
    remains: HashMap<usize, usize>,
    sent: HashMap<usize, usize>,
    points: Vec<ControlPoint>,
    freq_div: u32,
    start_idx: Option<u16>,
    finish_idx: Option<u16>,
}

impl FocusSTMOp {
    pub fn new(
        points: Vec<ControlPoint>,
        freq_div: u32,
        start_idx: Option<u16>,
        finish_idx: Option<u16>,
    ) -> Self {
        Self {
            points,
            remains: Default::default(),
            sent: Default::default(),
            freq_div,
            start_idx,
            finish_idx,
        }
    }
}

impl Operation for FocusSTMOp {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        assert!(self.remains[&device.idx()] > 0);

        tx[0] = TypeTag::FocusSTM as u8;

        let sent = self.sent[&device.idx()];
        let mut offset = std::mem::size_of::<TypeTag>()
            + std::mem::size_of::<FocusSTMControlFlags>()
            + std::mem::size_of::<u16>(); // size
        if sent == 0 {
            offset += std::mem::size_of::<u32>() // freq_div
            + std::mem::size_of::<u32>() // sound_speed
            + std::mem::size_of::<u16>() // start idx
            + std::mem::size_of::<u16>(); // finish idx
        }
        let send_bytes =
            ((self.points.len() - sent) * std::mem::size_of::<STMFocus>()).min(tx.len() - offset);
        let send_num = send_bytes / std::mem::size_of::<STMFocus>();
        assert!(send_num > 0);

        let mut f = FocusSTMControlFlags::NONE;
        f.set(FocusSTMControlFlags::STM_BEGIN, sent == 0);
        f.set(
            FocusSTMControlFlags::STM_END,
            sent + send_num == self.points.len(),
        );

        tx[2] = (send_num & 0xFF) as u8;
        tx[3] = (send_num >> 8) as u8;

        if sent == 0 {
            let freq_div = self.freq_div;
            tx[4] = (freq_div & 0xFF) as u8;
            tx[5] = ((freq_div >> 8) & 0xFF) as u8;
            tx[6] = ((freq_div >> 16) & 0xFF) as u8;
            tx[7] = ((freq_div >> 24) & 0xFF) as u8;

            let sound_speed = (device.sound_speed / METER * 1024.0).round() as u32;
            tx[8] = (sound_speed & 0xFF) as u8;
            tx[9] = ((sound_speed >> 8) & 0xFF) as u8;
            tx[10] = ((sound_speed >> 16) & 0xFF) as u8;
            tx[11] = ((sound_speed >> 24) & 0xFF) as u8;

            let start_idx = self.start_idx.unwrap_or(0);
            tx[12] = (start_idx & 0xFF) as u8;
            tx[13] = (start_idx >> 8) as u8;
            f.set(
                FocusSTMControlFlags::USE_START_IDX,
                self.start_idx.is_some(),
            );

            let finish_idx = self.finish_idx.unwrap_or(0);
            tx[14] = (finish_idx & 0xFF) as u8;
            tx[15] = (finish_idx >> 8) as u8;
            f.set(
                FocusSTMControlFlags::USE_FINISH_IDX,
                self.finish_idx.is_some(),
            );
        }
        tx[1] = f.bits();

        unsafe {
            let dst = std::slice::from_raw_parts_mut(
                tx[offset..].as_mut_ptr() as *mut STMFocus,
                send_num,
            );
            dst.iter_mut()
                .zip(self.points.iter().skip(sent).take(send_num))
                .try_for_each(|(d, p)| {
                    let lp = device.to_local(p.point());
                    d.set(lp.x, lp.y, lp.z, p.intensity())
                })?
        }

        self.sent.insert(device.idx(), sent + send_num);
        if sent == 0 {
            Ok(std::mem::size_of::<TypeTag>()
            + std::mem::size_of::<FocusSTMControlFlags>()
            + std::mem::size_of::<u16>() // size
            + std::mem::size_of::<u32>() // freq_div
            + std::mem::size_of::<u32>() // sound_speed
            + std::mem::size_of::<u16>() // start idx
            + std::mem::size_of::<u16>() // finish idx
            + std::mem::size_of::<STMFocus>() * send_num)
        } else {
            Ok(std::mem::size_of::<TypeTag>()
            + std::mem::size_of::<FocusSTMControlFlags>()
            + std::mem::size_of::<u16>() // size
            + std::mem::size_of::<STMFocus>() * send_num)
        }
    }

    fn required_size(&self, device: &Device) -> usize {
        if self.sent[&device.idx()] == 0 {
            std::mem::size_of::<TypeTag>()
                + std::mem::size_of::<FocusSTMControlFlags>()
                + std::mem::size_of::<u16>() // size
                + std::mem::size_of::<u32>() // freq_div
                + std::mem::size_of::<u32>() // sound_speed
                + std::mem::size_of::<u16>() // start idx
                + std::mem::size_of::<u16>() // finish idx
                + std::mem::size_of::<STMFocus>()
        } else {
            std::mem::size_of::<TypeTag>()
                + std::mem::size_of::<FocusSTMControlFlags>()
                + std::mem::size_of::<u16>() // size
                + std::mem::size_of::<STMFocus>()
        }
    }

    fn init(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if !(2..=FOCUS_STM_BUF_SIZE_MAX).contains(&self.points.len()) {
            return Err(AUTDInternalError::FocusSTMPointSizeOutOfRange(
                self.points.len(),
            ));
        }

        match self.start_idx {
            Some(idx) if idx >= self.points.len() as u16 => {
                return Err(AUTDInternalError::STMStartIndexOutOfRange)
            }
            _ => {}
        }
        match self.finish_idx {
            Some(idx) if idx >= self.points.len() as u16 => {
                return Err(AUTDInternalError::STMFinishIndexOutOfRange)
            }
            _ => {}
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

#[cfg(test)]
mod tests {
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
        const FRAME_SIZE: usize = 16 + 8 * FOCUS_STM_SIZE;

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

        let mut op = FocusSTMOp::new(points.clone(), freq_div, None, None);

        assert!(op.init(&geometry).is_ok());

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 16 + 8));

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
            assert_ne!(flag & FocusSTMControlFlags::STM_BEGIN.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::STM_END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                (FOCUS_STM_SIZE & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((FOCUS_STM_SIZE >> 8) & 0xFF) as u8
            );

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

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], 0x00);

            tx[FRAME_SIZE * dev.idx() + 16..]
                .chunks(std::mem::size_of::<STMFocus>())
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
        const FRAME_SIZE: usize = 30;
        const FOCUS_STM_SIZE: usize = (FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()
            + (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>() * 2;

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
        let mut op = FocusSTMOp::new(points.clone(), freq_div, None, None);

        assert!(op.init(&geometry).is_ok());

        // First frame
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 16 + 8));

        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.remains(dev), FOCUS_STM_SIZE));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(16
                    + (FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()
                        * std::mem::size_of::<STMFocus>())
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>() * 2
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & FocusSTMControlFlags::STM_BEGIN.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::STM_END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                (((FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((((FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()) >> 8) & 0xFF) as u8
            );

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

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], 0x00);

            tx[FRAME_SIZE * dev.idx() + 16..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(std::mem::size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .take((FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()),
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
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 4 + std::mem::size_of::<STMFocus>()));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(4 + (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()
                    * std::mem::size_of::<STMFocus>())
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.remains(dev),
                (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()
            )
        });

        geometry.devices().for_each(|dev| {
            assert_eq!(tx[dev.idx() * FRAME_SIZE], TypeTag::FocusSTM as u8);
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & FocusSTMControlFlags::STM_BEGIN.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::STM_END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                (((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()) >> 8) & 0xFF) as u8
            );

            tx[FRAME_SIZE * dev.idx() + 4..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(std::mem::size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip((FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>())
                        .take((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()),
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
        geometry
            .devices()
            .for_each(|dev| assert_eq!(op.required_size(dev), 4 + std::mem::size_of::<STMFocus>()));

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(
                    dev,
                    &mut tx[dev.idx() * FRAME_SIZE..(dev.idx() + 1) * FRAME_SIZE]
                ),
                Ok(4 + (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()
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
            assert_eq!(flag & FocusSTMControlFlags::STM_BEGIN.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::STM_END.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 2],
                (((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()) & 0xFF) as u8
            );
            assert_eq!(
                tx[dev.idx() * FRAME_SIZE + 3],
                ((((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()) >> 8) & 0xFF) as u8
            );

            tx[FRAME_SIZE * dev.idx() + 4..FRAME_SIZE * (dev.idx() + 1)]
                .chunks(std::mem::size_of::<STMFocus>())
                .zip(
                    points
                        .iter()
                        .skip(
                            (FRAME_SIZE - 16) / std::mem::size_of::<STMFocus>()
                                + (FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>(),
                        )
                        .take((FRAME_SIZE - 4) / std::mem::size_of::<STMFocus>()),
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
    fn focus_stm_op_idx() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = 16 + 8 * FOCUS_STM_SIZE;

        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE * NUM_DEVICE];

        let mut rng = rand::thread_rng();

        let start_idx = rng.gen_range(0..FOCUS_STM_SIZE as u16);
        let finish_idx = rng.gen_range(0..FOCUS_STM_SIZE as u16);

        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| ControlPoint::new(Vector3::zeros()))
            .collect();

        let mut op = FocusSTMOp::new(
            points.clone(),
            SAMPLING_FREQ_DIV_MIN,
            Some(start_idx),
            Some(finish_idx),
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], (start_idx & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], (start_idx >> 8) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], (finish_idx & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], (finish_idx >> 8) as u8);
        });

        let mut op = FocusSTMOp::new(points.clone(), SAMPLING_FREQ_DIV_MIN, Some(start_idx), None);

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_ne!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_eq!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], (start_idx & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], (start_idx >> 8) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], 0x00);
        });

        let mut op = FocusSTMOp::new(
            points.clone(),
            SAMPLING_FREQ_DIV_MIN,
            None,
            Some(finish_idx),
        );

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Ok(FRAME_SIZE)
            );
            op.commit(dev);
        });

        geometry.devices().for_each(|dev| {
            let flag = tx[dev.idx() * FRAME_SIZE + 1];
            assert_eq!(flag & FocusSTMControlFlags::USE_START_IDX.bits(), 0x00);
            assert_ne!(flag & FocusSTMControlFlags::USE_FINISH_IDX.bits(), 0x00);

            assert_eq!(tx[dev.idx() * FRAME_SIZE + 12], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 13], 0x00);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 14], (finish_idx & 0xFF) as u8);
            assert_eq!(tx[dev.idx() * FRAME_SIZE + 15], (finish_idx >> 8) as u8);
        });
    }

    #[test]
    fn focus_stm_op_buffer_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let test = |n: usize| {
            let mut rng = rand::thread_rng();

            let points: Vec<ControlPoint> = (0..n)
                .map(|_| {
                    ControlPoint::new(Vector3::new(
                        rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                        rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                        rng.gen_range(0.0 * MILLIMETER..500.0 * MILLIMETER),
                    ))
                    .with_intensity(rng.gen::<u8>())
                })
                .collect();
            let mut op = FocusSTMOp::new(points, SAMPLING_FREQ_DIV_MIN, None, None);
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

        let mut op = FocusSTMOp::new(points.clone(), freq_div, None, None);

        assert!(op.init(&geometry).is_ok());

        geometry.devices().for_each(|dev| {
            assert_eq!(
                op.pack(dev, &mut tx[dev.idx() * FRAME_SIZE..]),
                Err(AUTDInternalError::FocusSTMPointOutOfRange(x, x, x))
            );
        });
    }

    #[test]
    fn focus_stm_op_stm_idx_out_of_range() {
        let geometry = create_geometry(NUM_DEVICE, NUM_TRANS_IN_UNIT);

        let test = |n: usize, start_idx: Option<u16>, finish_idx: Option<u16>| {
            let mut rng = rand::thread_rng();

            let points: Vec<ControlPoint> = (0..n)
                .map(|_| {
                    ControlPoint::new(Vector3::new(
                        rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                        rng.gen_range(-500.0 * MILLIMETER..500.0 * MILLIMETER),
                        rng.gen_range(0.0 * MILLIMETER..500.0 * MILLIMETER),
                    ))
                    .with_intensity(rng.gen::<u8>())
                })
                .collect();

            let mut op =
                FocusSTMOp::new(points.clone(), SAMPLING_FREQ_DIV_MIN, start_idx, finish_idx);
            op.init(&geometry)
        };

        assert_eq!(test(10, Some(0), Some(0)), Ok(()));
        assert_eq!(test(10, Some(9), Some(0)), Ok(()));
        assert_eq!(
            test(10, Some(10), Some(0)),
            Err(AUTDInternalError::STMStartIndexOutOfRange)
        );

        assert_eq!(test(10, Some(0), Some(0)), Ok(()));
        assert_eq!(test(10, Some(0), Some(9)), Ok(()));
        assert_eq!(
            test(10, Some(0), Some(10)),
            Err(AUTDInternalError::STMFinishIndexOutOfRange)
        );
    }
}
