use super::super::{V12_1, operation::OperationGenerator};
use crate::{
    error::AUTDDriverError,
    firmware::driver::{Driver, Operation, TimerStrategy},
};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    link::Link,
    sleep::Sleep,
};

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) inner: crate::firmware::v12::transmission::Sender<'a, L, S, T>,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    /// Send the [`Datagram`] to the devices.
    pub fn send<'dev, 'tr, D: Datagram<'a, 'dev, 'tr>>(
        &mut self,
        s: D,
    ) -> Result<(), AUTDDriverError>
    where
        'a: 'dev,
        'dev: 'tr,
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator<'dev>,
        AUTDDriverError: From<<<D::G as OperationGenerator<'dev>>::O1 as Operation<'dev>>::Error>
            + From<<<D::G as OperationGenerator<'dev>>::O2 as Operation<'dev>>::Error>,
    {
        let timeout = self.inner.option.timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let mut g = s.operation_generator(
            self.inner.geometry,
            self.inner.env,
            &DeviceFilter::all_enabled(),
            &V12_1.firmware_limits(),
        )?;
        let mut operations = self
            .inner
            .geometry
            .iter()
            .map(|dev| g.generate(dev))
            .collect::<Vec<_>>();

        self.inner
            .send_impl(timeout, parallel_threshold, &mut operations)
    }
}
