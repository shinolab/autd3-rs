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
    firmware::FirmwareLimits,
    geometry::Geometry,
    link::Link,
    sleep::Sleep,
};

pub(crate) enum Inner<'a, L: Link, S: Sleep, T: TimerStrategy<S>> {
    V10(crate::firmware::v10::transmission::Sender<'a, L, S, T>),
    V11(crate::firmware::v11::transmission::Sender<'a, L, S, T>),
    V12(crate::firmware::v12::transmission::Sender<'a, L, S, T>),
    V12_1(crate::firmware::v12_1::transmission::Sender<'a, L, S, T>),
}

impl<'a, L: Link, S: Sleep, T: TimerStrategy<S>> Inner<'a, L, S, T> {
    fn option(&self) -> &SenderOption {
        match self {
            Inner::V10(inner) => &inner.option,
            Inner::V11(inner) => &inner.inner.option,
            Inner::V12(inner) => &inner.option,
            Inner::V12_1(inner) => &inner.inner.option,
        }
    }

    fn env(&self) -> &autd3_core::environment::Environment {
        match self {
            Inner::V10(inner) => inner.env,
            Inner::V11(inner) => inner.inner.env,
            Inner::V12(inner) => inner.env,
            Inner::V12_1(inner) => inner.inner.env,
        }
    }

    fn geometry(&self) -> &'a Geometry {
        match self {
            Inner::V10(inner) => inner.geometry,
            Inner::V11(inner) => inner.inner.geometry,
            Inner::V12(inner) => inner.geometry,
            Inner::V12_1(inner) => inner.inner.geometry,
        }
    }

    fn send_impl<O1, O2>(
        &mut self,
        timeout: Duration,
        parallel_threshold: usize,
        operations: &mut [Option<(O1, O2)>],
    ) -> Result<(), AUTDDriverError>
    where
        O1: Operation<'a>,
        O2: Operation<'a>,
        AUTDDriverError: From<O1::Error> + From<O2::Error>,
    {
        match self {
            Inner::V10(inner) => inner.send_impl(timeout, parallel_threshold, operations),
            Inner::V11(inner) => inner
                .inner
                .send_impl(timeout, parallel_threshold, operations),
            Inner::V12(inner) => inner.send_impl(timeout, parallel_threshold, operations),
            Inner::V12_1(inner) => inner
                .inner
                .send_impl(timeout, parallel_threshold, operations),
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
    pub fn send<D: Datagram<'a>>(&mut self, s: D) -> Result<(), AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator<'a>,
        AUTDDriverError: From<<<D::G as OperationGenerator<'a>>::O1 as Operation<'a>>::Error>
            + From<<<D::G as OperationGenerator<'a>>::O2 as Operation<'a>>::Error>,
    {
        let timeout = self.inner.option().timeout.unwrap_or(s.option().timeout);
        let parallel_threshold = s.option().parallel_threshold;

        let mut g = s.operation_generator(
            self.inner.geometry(),
            self.inner.env(),
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
