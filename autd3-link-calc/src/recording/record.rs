use std::time::Duration;

use autd3_driver::{
    defined::ULTRASOUND_PERIOD,
    derive::Builder,
    firmware::fpga::{Drive, SilencerTarget},
    geometry::Transducer,
};

use derive_more::Deref;

use crate::Calc;

#[derive(Debug, Clone, Builder)]
pub struct TransducerRecord {
    #[get]
    pub(crate) drive: Vec<Drive>,
    #[get]
    pub(crate) modulation: Vec<u8>,
}

#[derive(Debug, Clone, Deref, Builder)]
pub struct DeviceRecord {
    #[get]
    #[deref]
    pub(crate) records: Vec<TransducerRecord>,
}

#[derive(Debug, Clone, Deref)]
pub struct Record {
    #[deref]
    pub(crate) records: Vec<DeviceRecord>,
}

impl Calc {
    pub fn pulse_width(&self, record: &Record, tr: &Transducer) -> Vec<(Duration, u8)> {
        let record = &record.records[tr.dev_idx()][tr.idx()];
        let cpu = &self.sub_devices[tr.dev_idx()].cpu;
        match cpu.fpga().silencer_target() {
            SilencerTarget::Intensity => {
                let intensity = record
                    .drive
                    .iter()
                    .zip(record.modulation.iter())
                    .map(|(d, &m)| ((d.intensity().value() as u16 * m as u16) / 255) as u8)
                    .collect::<Vec<_>>();
                let intensity = cpu.fpga().apply_silencer(0, &intensity, false);
                intensity
                    .into_iter()
                    .enumerate()
                    .map(|(i, intensity)| {
                        (
                            i as u32 * ULTRASOUND_PERIOD,
                            cpu.fpga().pulse_width_encoder_table_at(intensity as _),
                        )
                    })
                    .collect()
            }
            SilencerTarget::PulseWidth => {
                let pulse_width = record
                    .drive
                    .iter()
                    .zip(record.modulation.iter())
                    .map(|(d, &m)| cpu.fpga().to_pulse_width(d.intensity(), m))
                    .collect::<Vec<_>>();
                cpu.fpga()
                    .apply_silencer(0, &pulse_width, false)
                    .into_iter()
                    .enumerate()
                    .map(|(i, pw)| (i as u32 * ULTRASOUND_PERIOD, pw))
                    .collect()
            }
        }
    }
}
