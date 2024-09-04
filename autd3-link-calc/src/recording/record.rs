use std::time::Duration;

use autd3_driver::{
    defined::ULTRASOUND_PERIOD,
    derive::Builder,
    ethercat::DcSysTime,
    firmware::fpga::{Drive, SilencerTarget},
};

use autd3_firmware_emulator::FPGAEmulator;
use derive_more::Deref;

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

#[derive(Builder)]
pub struct TransducerRecord<'a> {
    #[get]
    pub(crate) drive: Vec<Drive>,
    #[get]
    pub(crate) modulation: Vec<u8>,
    pub(crate) fpga: &'a FPGAEmulator,
}

#[derive(Deref)]
pub struct DeviceRecord<'a> {
    #[deref]
    pub(crate) records: Vec<TransducerRecord<'a>>,
}

#[derive(Deref, Builder)]
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
}
