use std::time::Duration;

use crate::{
    error::AUTDDriverError,
    firmware::{
        auto::operation::OperationGenerator,
        driver::{Operation, SenderOption, TimerStrategy, Version},
    },
};

use autd3_core::{
    datagram::{Datagram, DeviceFilter},
    derive::FirmwareLimits,
    geometry::Geometry,
    link::Link,
    sleep::Sleep,
};

pub(crate) enum Inner<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    V10(crate::firmware::v10::transmission::Sender<'a, L, S, T>),
    V11(crate::firmware::v11::transmission::Sender<'a, L, S, T>),
    V12(crate::firmware::v12::transmission::Sender<'a, L, S, T>),
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Inner<'a, L, S, T> {
    fn option(&self) -> &SenderOption {
        match self {
            Inner::V10(inner) => &inner.option,
            Inner::V11(inner) => &inner.inner.option,
            Inner::V12(inner) => &inner.option,
        }
    }

    fn geometry(&self) -> &Geometry {
        match self {
            Inner::V10(inner) => &inner.geometry,
            Inner::V11(inner) => &inner.inner.geometry,
            Inner::V12(inner) => &inner.geometry,
        }
    }

    fn send_impl<O1, O2>(
        &mut self,
        timeout: Duration,
        parallel_threshold: usize,
        operations: &mut [Option<(O1, O2)>],
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation,
        O2: Operation,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        match self {
            Inner::V10(inner) => inner.send_impl(timeout, parallel_threshold, operations),
            Inner::V11(inner) => inner
                .inner
                .send_impl(timeout, parallel_threshold, operations),
            Inner::V12(inner) => inner.send_impl(timeout, parallel_threshold, operations),
        }
    }
}

/// A struct to send the [`Datagram`] to the devices.
pub struct Sender<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    pub(crate) inner: Inner<'a, L, S, T>,
    pub(crate) version: Version,
    pub(crate) limits: FirmwareLimits,
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Sender<'a, L, S, T> {
    /// Send the [`Datagram`] to the devices.
    pub fn send<D: Datagram>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let timeout = self.inner.option().timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let mut g = s.operation_generator(
            self.inner.geometry(),
            &DeviceFilter::all_enabled(),
            &self.limits,
        )?;
        let mut operations = self
            .inner
            .geometry()
            .iter()
            .map(|dev| g.generate(dev, self.version))
            .collect::<Vec<_>>();

        self.inner
            .send_impl(timeout, parallel_threshold, &mut operations)
    }
}
