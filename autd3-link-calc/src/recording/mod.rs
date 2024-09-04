mod record;

use std::time::Duration;

use autd3_driver::{defined::ULTRASOUND_PERIOD, ethercat::DcSysTime};
pub use record::Record;
use record::{DeviceRecord, TransducerRecord};

use crate::{error::CalcError, Calc};

impl Calc {
    pub fn start_recording(&mut self) -> Result<(), CalcError> {
        self.start_record_at(DcSysTime::ZERO)
    }

    pub fn start_record_at(&mut self, start_time: DcSysTime) -> Result<(), CalcError> {
        if self.record.is_some() {
            return Err(CalcError::RecordingAlreadyStarted);
        }
        self.record = Some(Record {
            records: self
                .sub_devices
                .iter()
                .map(|sd| DeviceRecord {
                    records: sd
                        .device
                        .iter()
                        .map(|_| TransducerRecord {
                            drive: Vec::new(),
                            modulation: Vec::new(),
                        })
                        .collect(),
                })
                .collect(),
        });
        self.recording_tick = Some(start_time);
        Ok(())
    }

    pub fn finish_recording(&mut self) -> Result<Record, CalcError> {
        if self.record.is_none() {
            return Err(CalcError::RecodingNotStarted);
        }
        Ok(self.record.take().unwrap())
    }

    pub fn tick(&mut self, tick: Duration) -> Result<(), CalcError> {
        if let Some(record) = &mut self.record {
            if tick.is_zero() || tick.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
                return Err(CalcError::InvalidTick);
            }
            let mut t = self.recording_tick.unwrap();
            let end = t + tick;
            loop {
                self.sub_devices.iter_mut().for_each(|sd| {
                    sd.cpu.update_with_sys_time(t);
                    let m = sd.cpu.fpga().modulation();
                    let d = sd.cpu.fpga().drives();
                    sd.device.iter().for_each(|tr| {
                        record.records[tr.dev_idx()].records[tr.idx()]
                            .drive
                            .push(d[tr.idx()]);
                        record.records[tr.dev_idx()].records[tr.idx()]
                            .modulation
                            .push(m);
                    });
                });
                t = t + ULTRASOUND_PERIOD;
                if t == end {
                    break;
                }
            }
            self.recording_tick = Some(end);
            Ok(())
        } else {
            Err(CalcError::RecodingNotStarted)
        }
    }
}
