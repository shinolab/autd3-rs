use std::time::Duration;

use super::Datagram;
use crate::{
    derive::{AUTDInternalError, Device, Geometry},
    firmware::{
        fpga::{Segment, TransitionMode},
        operation::Operation,
    },
};

pub struct DatagramWithSegmentTransition<'a, D: DatagramST<'a>> {
    datagram: D,
    segment: Segment,
    transition_mode: Option<TransitionMode>,
    _phantom: std::marker::PhantomData<&'a D>,
}

impl<'a, D: DatagramST<'a>> DatagramWithSegmentTransition<'a, D> {
    pub const fn segment(&self) -> Segment {
        self.segment
    }
}

impl<'a, D: DatagramST<'a>> DatagramWithSegmentTransition<'a, D> {
    pub const fn transition_mode(&self) -> Option<TransitionMode> {
        self.transition_mode
    }
}

impl<'a, D: DatagramST<'a>> std::ops::Deref for DatagramWithSegmentTransition<'a, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.datagram
    }
}

impl<'a, D: DatagramST<'a>> Datagram<'a> for DatagramWithSegmentTransition<'a, D> {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        self.datagram
            .operation_with_segment(geometry, self.segment, self.transition_mode)
    }

    fn timeout(&self) -> Option<Duration> {
        self.datagram.timeout()
    }
}

impl<'a, D: DatagramST<'a>> Datagram<'a> for D {
    type O1 = D::O1;
    type O2 = D::O2;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        self.operation_with_segment(geometry, Segment::S0, Some(TransitionMode::Immediate))
    }

    fn timeout(&self) -> Option<Duration> {
        <Self as DatagramST>::timeout(self)
    }
}

pub trait DatagramST<'a> {
    type O1: Operation + 'a;
    type O2: Operation + 'a;

    fn operation_with_segment(
        &'a self,
        geometry: &'a Geometry,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

pub trait IntoDatagramWithSegmentTransition<'a, D: DatagramST<'a>> {
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegmentTransition<'a, D>;
}

impl<'a, D: DatagramST<'a>> IntoDatagramWithSegmentTransition<'a, D> for D {
    fn with_segment(
        self,
        segment: Segment,
        transition_mode: Option<TransitionMode>,
    ) -> DatagramWithSegmentTransition<'a, D> {
        DatagramWithSegmentTransition {
            datagram: self,
            segment,
            transition_mode,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, D: DatagramST<'a> + Clone> Clone for DatagramWithSegmentTransition<'a, D> {
    fn clone(&self) -> Self {
        Self {
            datagram: self.datagram.clone(),
            segment: self.segment,
            transition_mode: self.transition_mode,
            _phantom: std::marker::PhantomData,
        }
    }
}
