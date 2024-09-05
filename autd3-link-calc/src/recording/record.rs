use std::time::Duration;

use autd3_driver::{
    defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT},
    derive::Builder,
    ethercat::DcSysTime,
    firmware::fpga::{Drive, SilencerTarget},
};

use autd3_firmware_emulator::FPGAEmulator;
use derive_more::{Debug, Deref};

pub(crate) struct RawTransducerRecord {
    pub drive: Vec<Drive>,
    pub modulation: Vec<u8>,
}

pub(crate) struct RawDeviceRecord {
    pub(crate) records: Vec<RawTransducerRecord>,
}

pub(crate) struct RawRecord {
    pub records: Vec<RawDeviceRecord>,
    pub start: DcSysTime,
    pub current: DcSysTime,
}

#[derive(Builder, Debug)]
pub struct TransducerRecord<'a> {
    #[get]
    pub(crate) drive: Vec<Drive>,
    #[get]
    pub(crate) modulation: Vec<u8>,
    #[debug(skip)]
    pub(crate) fpga: &'a FPGAEmulator,
}

#[derive(Deref, Debug)]
pub struct DeviceRecord<'a> {
    #[deref]
    pub(crate) records: Vec<TransducerRecord<'a>>,
}

#[derive(Deref, Builder, Debug)]
pub struct Record<'a> {
    #[deref]
    pub(crate) records: Vec<DeviceRecord<'a>>,
    #[get]
    pub(crate) start: DcSysTime,
    #[get]
    pub(crate) end: DcSysTime,
}

impl<'a> TransducerRecord<'a> {
    pub fn pulse_width(&self) -> Vec<(Duration, u8)> {
        let fpga = &self.fpga;
        match fpga.silencer_target() {
            SilencerTarget::Intensity => {
                let intensity = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| ((d.intensity().value() as u16 * m as u16) / 255) as u8)
                    .collect::<Vec<_>>();
                let intensity = fpga.apply_silencer(0, &intensity, false);
                intensity
                    .into_iter()
                    .enumerate()
                    .map(|(i, intensity)| {
                        (
                            i as u32 * ULTRASOUND_PERIOD,
                            fpga.pulse_width_encoder_table_at(intensity as _),
                        )
                    })
                    .collect()
            }
            SilencerTarget::PulseWidth => {
                let pulse_width = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| fpga.to_pulse_width(d.intensity(), m))
                    .collect::<Vec<_>>();
                fpga.apply_silencer(0, &pulse_width, false)
                    .into_iter()
                    .enumerate()
                    .map(|(i, pw)| (i as u32 * ULTRASOUND_PERIOD, pw))
                    .collect()
            }
        }
    }

    pub fn output_voltage(&self) -> Vec<(f32, f32)> {
        self.pulse_width()
            .into_iter()
            .zip(self.drive.iter())
            .flat_map(|((t, pw), d)| {
                let rise = d.phase().value().wrapping_sub(pw / 2);
                let fall = d.phase().value().wrapping_add(pw / 2 + (pw & 0x01));
                (0..=255u8).map(move |i| {
                    const TS: f32 =
                        1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);
                    (
                        t.as_secs_f32() + i as f32 * TS,
                        #[allow(clippy::collapsible_else_if)]
                        if rise <= fall {
                            if (rise <= i) && (i < fall) {
                                12.0
                            } else {
                                -12.0
                            }
                        } else {
                            if (i < fall) || (rise <= i) {
                                12.0
                            } else {
                                -12.0
                            }
                        },
                    )
                })
            })
            .collect()
    }
}
