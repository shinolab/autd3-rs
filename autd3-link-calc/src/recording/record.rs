use autd3_driver::{
    defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT},
    derive::Builder,
    ethercat::DcSysTime,
    firmware::fpga::{Drive, SilencerTarget},
};

use autd3_firmware_emulator::FPGAEmulator;
use derive_more::{Debug, Deref};
use polars::prelude::*;

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
    pub(crate) drive: Vec<Drive>,
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
    fn time(n: usize) -> Series {
        (0..n)
            .map(|i| (i as u32 * ULTRASOUND_PERIOD).as_secs_f32())
            .collect()
    }

    pub fn drive(&self) -> DataFrame {
        let time = Self::time(self.drive.len());
        let phase = self
            .drive
            .iter()
            .map(|d| d.phase().value())
            .collect::<Vec<_>>();
        let intensity = self
            .drive
            .iter()
            .map(|d| d.intensity().value())
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &time,
            "phase" => &phase,
            "intensity" => &intensity
        )
        .unwrap()
    }

    pub fn modulation(&self) -> DataFrame {
        let time = Self::time(self.modulation.len());
        df!(
            "time[s]" => &time,
            "modulation" => &self
            .modulation
        )
        .unwrap()
    }

    fn pulse_width_after_silencer(&self) -> Vec<u8> {
        match self.fpga.silencer_target() {
            SilencerTarget::Intensity => {
                let intensity = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| ((d.intensity().value() as u16 * m as u16) / 255) as u8)
                    .collect::<Vec<_>>();
                self.fpga
                    .apply_silencer(0, &intensity, false)
                    .into_iter()
                    .map(|intensity| self.fpga.pulse_width_encoder_table_at(intensity as _))
                    .collect::<Vec<_>>()
            }
            SilencerTarget::PulseWidth => {
                let pulse_width = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| self.fpga.to_pulse_width(d.intensity(), m))
                    .collect::<Vec<_>>();
                self.fpga.apply_silencer(0, &pulse_width, false)
            }
        }
    }

    pub fn pulse_width(&self) -> DataFrame {
        let pulse_width = self.pulse_width_after_silencer();
        let time = Self::time(pulse_width.len());
        df!(
            "time[s]" => &time,
            "pulsewidth" => &pulse_width
        )
        .unwrap()
    }

    pub fn output_voltage(&self) -> DataFrame {
        const V: f32 = 12.0;
        let time = (0..self.modulation.len())
            .map(|i| i as u32 * ULTRASOUND_PERIOD)
            .flat_map(|t| {
                (0..=255u8).map(move |i| {
                    const TS: f32 =
                        1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);
                    t.as_secs_f32() + i as f32 * TS
                })
            })
            .collect::<Vec<_>>();
        let v = self
            .pulse_width_after_silencer()
            .into_iter()
            .zip(self.drive.iter())
            .flat_map(|(pw, d)| {
                let rise = d.phase().value().wrapping_sub(pw / 2);
                let fall = d.phase().value().wrapping_add(pw / 2 + (pw & 0x01));
                (0..=255u8).map(move |i| {
                    #[allow(clippy::collapsible_else_if)]
                    if rise <= fall {
                        if (rise <= i) && (i < fall) {
                            V
                        } else {
                            -V
                        }
                    } else {
                        if (i < fall) || (rise <= i) {
                            V
                        } else {
                            -V
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &time,
            "voltage[V]" => &v
        )
        .unwrap()
    }
}
