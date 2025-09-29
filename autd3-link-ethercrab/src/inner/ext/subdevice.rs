use ethercrab::{Command, MainDevice, RegisterAddress, SubDevicePdi};

use super::state::State;

pub(crate) trait SubDeviceExt {
    async fn read_state(&self) -> Result<State, ethercrab::error::Error>;
    async fn write_state(
        &self,
        main_device: &MainDevice,
        state: State,
    ) -> Result<u16, ethercrab::error::Error>;
}

impl<const N: usize> SubDeviceExt for ethercrab::SubDeviceRef<'_, SubDevicePdi<'_, N>> {
    async fn read_state(&self) -> Result<State, ethercrab::error::Error> {
        self.register_read::<u16>(RegisterAddress::AlStatus)
            .await
            .map(State::from)
    }

    async fn write_state(
        &self,
        main_device: &MainDevice<'_>,
        state: State,
    ) -> Result<u16, ethercrab::error::Error> {
        Command::fpwr(self.configured_address(), RegisterAddress::AlControl.into())
            .send_receive::<u16>(main_device, state.state())
            .await
    }
}
