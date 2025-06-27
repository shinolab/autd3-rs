use autd3_core::{link::Link, sleep::Sleep};

use crate::{
    datagram::{
        Clear, FixedCompletionSteps, Silencer,
        implements::{Null, Static},
    },
    firmware::{
        driver::{Driver, Sender, TimerStrategy},
        version::FirmwareVersion,
    },
};

use super::fpga::{
    FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_WIDTH,
    FOCI_STM_FOCI_NUM_MAX, GAIN_STM_BUF_SIZE_MAX, MOD_BUF_SIZE_MAX,
};

/// A driver for firmware version 12.
pub struct V12;

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for super::transmission::Sender<'a, L, S, T>
{
    fn initialize_devices(mut self) -> Result<(), crate::error::AUTDDriverError> {
        // If the device is used continuously without powering off, the first data may be ignored because the first msg_id equals to the remaining msg_id in the device.
        // Therefore, send a meaningless data.
        self.send(crate::datagram::Nop)?;

        self.send((
            crate::datagram::Clear::new(),
            crate::datagram::Synchronize::new(),
        ))
    }

    fn firmware_version(
        mut self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        use crate::{
            datagram::FirmwareVersionType::*,
            firmware::version::{CPUVersion, FPGAVersion, Major, Minor},
        };

        let cpu_major = self.fetch_firminfo(CPUMajor)?;
        let cpu_minor = self.fetch_firminfo(CPUMinor)?;
        let fpga_major = self.fetch_firminfo(FPGAMajor)?;
        let fpga_minor = self.fetch_firminfo(FPGAMinor)?;
        let fpga_functions = self.fetch_firminfo(FPGAFunctions)?;
        self.fetch_firminfo(Clear)?;

        Ok(self
            .geometry
            .iter()
            .map(|dev| FirmwareVersion {
                idx: dev.idx(),
                cpu: CPUVersion {
                    major: Major(cpu_major[dev.idx()]),
                    minor: Minor(cpu_minor[dev.idx()]),
                },
                fpga: FPGAVersion {
                    major: Major(fpga_major[dev.idx()]),
                    minor: Minor(fpga_minor[dev.idx()]),
                    function_bits: fpga_functions[dev.idx()],
                },
            })
            .collect())
    }

    fn close(mut self) -> Result<(), crate::error::AUTDDriverError> {
        [
            self.send(Silencer {
                config: FixedCompletionSteps {
                    strict: false,
                    ..Default::default()
                },
            }),
            self.send((Static::default(), Null)),
            self.send(Clear {}),
            Ok(self.link.close()?),
        ]
        .into_iter()
        .try_fold((), |_, x| x)
    }
}

impl Driver for V12 {
    type Sender<'a, L, S, T>
        = super::transmission::Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>;
    type FPGAState = super::fpga::FPGAState;

    fn new() -> Self {
        Self
    }

    fn firmware_limits(&self) -> autd3_core::derive::FirmwareLimits {
        autd3_core::derive::FirmwareLimits {
            mod_buf_size_max: MOD_BUF_SIZE_MAX as _,
            gain_stm_buf_size_max: GAIN_STM_BUF_SIZE_MAX as _,
            foci_stm_buf_size_max: FOCI_STM_BUF_SIZE_MAX as _,
            num_foci_max: FOCI_STM_FOCI_NUM_MAX as _,
            foci_stm_fixed_num_unit: FOCI_STM_FIXED_NUM_UNIT,
            foci_stm_fixed_num_width: FOCI_STM_FIXED_NUM_WIDTH as _,
        }
    }

    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::derive::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            msg_id,
            link,
            geometry,
            sent_flags,
            rx,
            option,
            timer_strategy,
            _phantom: std::marker::PhantomData,
        }
    }
}
