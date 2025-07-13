use crate::{
    error::AUTDDriverError,
    firmware::{
        driver::{
            Operation,
            r#async::{Driver, TimerStrategy},
        },
        v12_1::{V12_1, operation::OperationGenerator},
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    link::AsyncLink,
    sleep::r#async::Sleep,
};

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) inner: crate::firmware::v12::r#async::sender::Sender<'a, L, S, T>,
}

impl<'a, L: AsyncLink, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    /// Send the [`Datagram`] to the devices.
    pub async fn send<'dev, 'tr, D: Datagram<'a, 'dev, 'tr>>(
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
            .await
    }
}
