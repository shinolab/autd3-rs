use autd3_core::{environment::Environment, link::Link, sleep::Sleep};

use crate::firmware::driver::{Driver, Sender, TimerStrategy};

use super::fpga::{
    FOCI_STM_BUF_SIZE_MAX, FOCI_STM_FIXED_NUM_UNIT, FOCI_STM_FIXED_NUM_WIDTH,
    FOCI_STM_FOCI_NUM_MAX, GAIN_STM_BUF_SIZE_MAX, MOD_BUF_SIZE_MAX, ULTRASOUND_PERIOD_COUNT_BITS,
};

/// A driver for firmware version 11.
pub struct V11;

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T>
    for super::transmission::Sender<'a, L, S, T>
{
    fn initialize_devices(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.initialize_devices()
    }

    fn firmware_version(
        self,
    ) -> Result<Vec<crate::firmware::version::FirmwareVersion>, crate::error::AUTDDriverError> {
        self.inner.firmware_version()
    }

    fn close(self) -> Result<(), crate::error::AUTDDriverError> {
        self.inner.close()
    }
}

impl Driver for V11 {
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

    fn firmware_limits(&self) -> autd3_core::datagram::FirmwareLimits {
        autd3_core::datagram::FirmwareLimits {
            mod_buf_size_max: MOD_BUF_SIZE_MAX as _,
            gain_stm_buf_size_max: GAIN_STM_BUF_SIZE_MAX as _,
            foci_stm_buf_size_max: FOCI_STM_BUF_SIZE_MAX as _,
            num_foci_max: FOCI_STM_FOCI_NUM_MAX as _,
            foci_stm_fixed_num_unit: FOCI_STM_FIXED_NUM_UNIT,
            foci_stm_fixed_num_width: FOCI_STM_FIXED_NUM_WIDTH as _,
            ultrasound_period: 1 << ULTRASOUND_PERIOD_COUNT_BITS as u32,
        }
    }

    fn sender<'a, L, S, T>(
        &self,
        msg_id: &'a mut autd3_core::link::MsgId,
        link: &'a mut L,
        geometry: &'a autd3_core::geometry::Geometry,
        sent_flags: &'a mut [bool],
        rx: &'a mut [autd3_core::link::RxMessage],
        env: &'a Environment,
        option: crate::firmware::driver::SenderOption,
        timer_strategy: T,
    ) -> Self::Sender<'a, L, S, T>
    where
        L: autd3_core::link::Link + 'a,
        S: autd3_core::sleep::Sleep,
        T: TimerStrategy<S>,
    {
        Self::Sender {
            inner: crate::firmware::v10::transmission::Sender {
                msg_id,
                link,
                geometry,
                sent_flags,
                rx,
                env,
                option,
                timer_strategy,
                _phantom: std::marker::PhantomData,
            },
        }
    }
}
