use std::time::Duration;

use crate::{
    datagram::*,
    derive::{AUTDInternalError, Device, Geometry, Segment, TransitionMode},
    firmware::operation::SwapSegmentOp,
};

use super::OperationGenerator;

#[derive(Debug, Clone, Copy)]
pub enum SwapSegment {
    Gain(Segment),
    Modulation(Segment, TransitionMode),
    FociSTM(Segment, TransitionMode),
    GainSTM(Segment, TransitionMode),
}

pub struct SwapSegmentOpGenerator {
    segment: SwapSegment,
}

impl OperationGenerator for SwapSegmentOpGenerator {
    type O1 = SwapSegmentOp;
    type O2 = NullOp;

    fn generate(&self, _: &Device) -> (Self::O1, Self::O2) {
        (Self::O1::new(self.segment), Self::O2::default())
    }
}

impl Datagram for SwapSegment {
    type G = SwapSegmentOpGenerator;

    fn timeout(&self) -> Option<Duration> {
        Some(DEFAULT_TIMEOUT)
    }

    fn operation_generator(self, _: &Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(SwapSegmentOpGenerator { segment: self })
    }

    fn parallel_threshold(&self) -> Option<usize> {
        Some(usize::MAX)
    }

    #[tracing::instrument(skip(_geometry))]
    // GRCOV_EXCL_START
    fn trace(&self, _geometry: &Geometry) {
        tracing::debug!("{}", tynm::type_name::<Self>());
    }
    // GRCOV_EXCL_STOP
}

#[cfg(feature = "capi")]
impl Default for SwapSegment {
    fn default() -> Self {
        Self::Gain(Segment::default())
    }
}
