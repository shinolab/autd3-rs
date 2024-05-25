use crate::{
    defined::{ControlPoint, METER},
    error::AUTDInternalError,
    firmware::{
        fpga::{STMFocus, Segment, TransitionMode, TRANSITION_MODE_NONE},
        operation::{cast, Operation, TypeTag},
    },
    geometry::Device,
};

use super::FocusSTMControlFlags;

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

pub struct FocusSTMOp<F: Fn(usize) -> ControlPoint> {
    remains: usize,
    points: F,
    size: usize,
    freq_div: u32,
    rep: u32,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
}

impl<F: Fn(usize) -> ControlPoint> FocusSTMOp<F> {
    pub fn new(
        points: F,
        size: usize,
        freq_div: u32,
        rep: u32,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Self {
        Self {
            points,
            size,
            remains: size,
            freq_div,
            rep,
            segment,
            transition_mode,
        }
    }
}

impl<F: Fn(usize) -> ControlPoint> Operation for FocusSTMOp<F> {
    fn pack(&mut self, device: &Device, tx: &mut [u8]) -> Result<usize, AUTDInternalError> {
        let sent = self.size - self.remains;

        let offset = if sent == 0 {
            std::mem::size_of::<FocusSTMHead>()
        } else {
            std::mem::size_of::<FocusSTMSubseq>()
        };

        let send_bytes = (self.remains * std::mem::size_of::<STMFocus>()).min(tx.len() - offset);
        let send_num = send_bytes / std::mem::size_of::<STMFocus>();
        assert!(send_num > 0);

        if sent == 0 {
            *cast::<FocusSTMHead>(tx) = FocusSTMHead {
                tag: TypeTag::FocusSTM,
                flag: FocusSTMControlFlags::BEGIN,
                transition_mode: self
                    .transition_mode
                    .map(|m| m.mode())
                    .unwrap_or(TRANSITION_MODE_NONE),
                transition_value: self.transition_mode.map(|m| m.value()).unwrap_or(0),
                send_num: send_num as u8,
                freq_div: self.freq_div,
                sound_speed: (device.sound_speed / METER
                    * 1024.0
                    * crate::defined::FREQ_40K.hz() as f64
                    / device.ultrasound_freq().hz() as f64)
                    .round() as u32,
                rep: self.rep,
            };
        } else {
            *cast::<FocusSTMSubseq>(tx) = FocusSTMSubseq {
                tag: TypeTag::FocusSTM,
                flag: FocusSTMControlFlags::NONE,
                send_num: send_num as u8,
            };
        }

        cast::<FocusSTMSubseq>(tx)
            .flag
            .set(FocusSTMControlFlags::SEGMENT, self.segment == Segment::S1);

        if sent + send_num == self.size {
            let d = cast::<FocusSTMSubseq>(tx);
            d.flag.set(FocusSTMControlFlags::END, true);
            d.flag.set(
                FocusSTMControlFlags::TRANSITION,
                self.transition_mode.is_some(),
            );
        }

        (0..send_num).try_for_each(|i| {
            let p = (self.points)(sent + i);
            let lp = device.to_local(p.point());
            cast::<STMFocus>(&mut tx[offset + i * std::mem::size_of::<STMFocus>()..]).set(
                lp.x,
                lp.y,
                lp.z,
                p.intensity(),
            )
        })?;

        self.remains -= send_num;
        if sent == 0 {
            Ok(std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>() * send_num)
        } else {
            Ok(std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>() * send_num)
        }
    }

    fn required_size(&self, _: &Device) -> usize {
        if self.remains == self.size {
            std::mem::size_of::<FocusSTMHead>() + std::mem::size_of::<STMFocus>()
        } else {
            std::mem::size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
        }
    }

    fn is_done(&self) -> bool {
        self.remains == 0
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use rand::prelude::*;

    use super::*;
    use crate::{
        defined::mm,
        ethercat::DcSysTime,
        firmware::{
            fpga::{
                FOCUS_STM_FIXED_NUM_UNIT, FOCUS_STM_FIXED_NUM_UPPER_X, SAMPLING_FREQ_DIV_MAX,
                SAMPLING_FREQ_DIV_MIN,
            },
            operation::tests::parse_tx_as,
        },
        geometry::{tests::create_device, Vector3},
    };

    const NUM_TRANS_IN_UNIT: usize = 249;

    #[test]
    fn test() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize =
            size_of::<FocusSTMHead>() + size_of::<STMFocus>() * FOCUS_STM_SIZE;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| {
                ControlPoint::new(Vector3::new(
                    rng.gen_range(-500.0 * mm..500.0 * mm),
                    rng.gen_range(-500.0 * mm..500.0 * mm),
                    rng.gen_range(0.0 * mm..500.0 * mm),
                ))
                .with_intensity(rng.gen::<u8>())
            })
            .collect();
        let rep = 0xFFFFFFFF;
        let segment = Segment::S0;
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let transition_value = 0x0123456789ABCDEF;
        let transition_mode = TransitionMode::SysTime(
            DcSysTime::from_utc(
                time::macros::datetime!(2000-01-01 0:00 UTC)
                    + std::time::Duration::from_nanos(transition_value),
            )
            .unwrap(),
        );

        let mut op = FocusSTMOp::new(
            {
                let points = points.clone();
                move |i| points[i]
            },
            FOCUS_STM_SIZE,
            freq_div,
            rep,
            segment,
            Some(transition_mode),
        );

        assert_eq!(
            op.required_size(&device),
            size_of::<FocusSTMHead>() + size_of::<STMFocus>()
        );

        assert_eq!(op.remains, FOCUS_STM_SIZE);

        assert_eq!(op.pack(&device, &mut tx), Ok(FRAME_SIZE));

        assert_eq!(op.remains, 0);

        assert_eq!(TypeTag::FocusSTM as u8, tx[0]);
        assert_eq!(
            (FocusSTMControlFlags::BEGIN
                | FocusSTMControlFlags::END
                | FocusSTMControlFlags::TRANSITION)
                .bits(),
            tx[offset_of!(FocusSTMHead, flag)]
        );
        assert_eq!(FOCUS_STM_SIZE as u8, tx[offset_of!(FocusSTMHead, send_num)]);
        assert_eq!(
            freq_div,
            parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, freq_div)..])
        );
        let sound_speed = (device.sound_speed / METER * 1024.0).round() as u32;
        assert_eq!(
            sound_speed,
            parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, sound_speed)..])
        );
        assert_eq!(
            rep,
            parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, rep)..])
        );
        assert_eq!(
            transition_mode.mode(),
            tx[offset_of!(FocusSTMHead, transition_mode)]
        );
        assert_eq!(
            ((transition_value / crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC) + 1)
                * crate::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC,
            parse_tx_as::<u64>(&tx[offset_of!(FocusSTMHead, transition_value)..])
        );
        tx[size_of::<FocusSTMHead>()..]
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
            });
    }

    #[test]
    fn test_div() {
        const FRAME_SIZE: usize = 32;
        const FOCUS_STM_SIZE: usize = (FRAME_SIZE - size_of::<FocusSTMHead>())
            / size_of::<STMFocus>()
            + (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / size_of::<STMFocus>() * 2;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let mut rng = rand::thread_rng();

        let points: Vec<ControlPoint> = (0..FOCUS_STM_SIZE)
            .map(|_| {
                ControlPoint::new(Vector3::new(
                    rng.gen_range(-500.0 * mm..500.0 * mm),
                    rng.gen_range(-500.0 * mm..500.0 * mm),
                    rng.gen_range(0.0 * mm..500.0 * mm),
                ))
                .with_intensity(rng.gen::<u8>())
            })
            .collect();
        let freq_div: u32 = rng.gen_range(SAMPLING_FREQ_DIV_MIN..SAMPLING_FREQ_DIV_MAX);
        let rep = rng.gen_range(0x0000001..=0xFFFFFFFF);
        let segment = Segment::S1;

        let mut op = FocusSTMOp::new(
            {
                let points = points.clone();
                move |i| points[i]
            },
            FOCUS_STM_SIZE,
            freq_div,
            rep,
            segment,
            None,
        );

        // First frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FocusSTMHead>() + size_of::<STMFocus>()
            );

            assert_eq!(op.remains, FOCUS_STM_SIZE);

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<FocusSTMHead>()
                    + (FRAME_SIZE - size_of::<FocusSTMHead>()) / size_of::<STMFocus>()
                        * size_of::<STMFocus>())
            );

            assert_eq!(
                op.remains,
                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>() * 2
            );

            assert_eq!(TypeTag::FocusSTM as u8, tx[0]);
            assert_eq!(
                (FocusSTMControlFlags::BEGIN | FocusSTMControlFlags::SEGMENT).bits(),
                tx[offset_of!(FocusSTMHead, flag)]
            );
            assert_eq!(
                ((FRAME_SIZE - size_of::<FocusSTMHead>()) / std::mem::size_of::<STMFocus>()) as u8,
                tx[offset_of!(FocusSTMHead, send_num)],
            );
            assert_eq!(tx[4], (freq_div & 0xFF) as u8);
            assert_eq!(
                freq_div,
                parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, freq_div)..])
            );
            let sound_speed = (device.sound_speed / METER * 1024.0).round() as u32;
            assert_eq!(
                sound_speed,
                parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, sound_speed)..])
            );
            assert_eq!(
                rep,
                parse_tx_as::<u32>(&tx[offset_of!(FocusSTMHead, rep)..])
            );

            tx[size_of::<FocusSTMHead>()..]
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
                });
        }

        // Second frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
            );

            assert_eq!(
                op.pack(&device, &mut tx),
                Ok(size_of::<FocusSTMSubseq>()
                    + (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
                        * std::mem::size_of::<STMFocus>())
            );

            assert_eq!(
                op.remains,
                (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
            );

            assert_eq!(TypeTag::FocusSTM as u8, tx[0]);
            assert_eq!(
                FocusSTMControlFlags::SEGMENT.bits(),
                tx[offset_of!(FocusSTMHead, flag)]
            );
            assert_eq!(
                ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                    as u8,
                tx[offset_of!(FocusSTMHead, send_num)],
            );
            tx[size_of::<FocusSTMSubseq>()..]
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
                });
        }

        // Final frame
        {
            assert_eq!(
                op.required_size(&device),
                size_of::<FocusSTMSubseq>() + std::mem::size_of::<STMFocus>()
            );

            assert_eq!(
                op.pack(&device, &mut tx[0..(&device.idx() + 1) * FRAME_SIZE]),
                Ok(size_of::<FocusSTMSubseq>()
                    + (FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>()
                        * std::mem::size_of::<STMFocus>())
            );

            assert_eq!(op.remains, 0);

            assert_eq!(TypeTag::FocusSTM as u8, tx[0]);
            assert_eq!(
                (FocusSTMControlFlags::SEGMENT | FocusSTMControlFlags::END).bits(),
                tx[offset_of!(FocusSTMHead, flag)]
            );
            assert_eq!(
                ((FRAME_SIZE - size_of::<FocusSTMSubseq>()) / std::mem::size_of::<STMFocus>())
                    as u8,
                tx[offset_of!(FocusSTMHead, send_num)],
            );
            tx[size_of::<FocusSTMSubseq>()..]
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
        }
    }

    #[test]
    fn test_point_out_of_range() {
        const FOCUS_STM_SIZE: usize = 100;
        const FRAME_SIZE: usize = 16 + 8 * FOCUS_STM_SIZE;

        let device = create_device(0, NUM_TRANS_IN_UNIT);

        let mut tx = vec![0x00u8; FRAME_SIZE];

        let x = FOCUS_STM_FIXED_NUM_UNIT * (FOCUS_STM_FIXED_NUM_UPPER_X as f64 + 1.);

        let mut op = FocusSTMOp::new(
            |_| ControlPoint::new(Vector3::new(x, x, x)).with_intensity(0),
            FOCUS_STM_SIZE,
            SAMPLING_FREQ_DIV_MIN,
            0xFFFFFFFF,
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );

        assert_eq!(
            op.pack(&device, &mut tx),
            Err(AUTDInternalError::FocusSTMPointOutOfRange(x, x, x))
        );
    }
}
