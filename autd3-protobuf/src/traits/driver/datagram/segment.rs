use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::datagram::SwapSegment {
    type Message = SwapSegment;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            inner: Some(match self {
                autd3_driver::datagram::SwapSegment::Gain(segment, transition) => {
                    swap_segment::Inner::Gain(SwapSegmentGain {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::Modulation(segment, transition) => {
                    swap_segment::Inner::Modulation(SwapSegmentModulation {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::FociSTM(segment, transition) => {
                    swap_segment::Inner::FociStm(SwapSegmentFociStm {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::GainSTM(segment, transition) => {
                    swap_segment::Inner::GainStm(SwapSegmentGainStm {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
            }),
        })
    }
}

impl FromMessage<SwapSegment> for autd3_driver::datagram::SwapSegment {
    fn from_msg(msg: &SwapSegment) -> Result<Self, AUTDProtoBufError> {
        Ok(match msg.inner.ok_or(AUTDProtoBufError::DataParseError)? {
            swap_segment::Inner::Gain(inner) => {
                let mode = inner
                    .transition_mode
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?;
                autd3_driver::datagram::SwapSegment::Gain(
                    autd3_driver::firmware::fpga::Segment::from(Segment::try_from(inner.segment)?),
                    autd3_driver::firmware::fpga::TransitionMode::from_msg(mode)?,
                )
            }
            swap_segment::Inner::Modulation(inner) => {
                let mode = inner
                    .transition_mode
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?;
                autd3_driver::datagram::SwapSegment::Modulation(
                    autd3_driver::firmware::fpga::Segment::from(Segment::try_from(inner.segment)?),
                    autd3_driver::firmware::fpga::TransitionMode::from_msg(mode)?,
                )
            }
            swap_segment::Inner::FociStm(inner) => {
                let mode = inner
                    .transition_mode
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?;
                autd3_driver::datagram::SwapSegment::FociSTM(
                    autd3_driver::firmware::fpga::Segment::from(Segment::try_from(inner.segment)?),
                    autd3_driver::firmware::fpga::TransitionMode::from_msg(mode)?,
                )
            }
            swap_segment::Inner::GainStm(inner) => {
                let mode = inner
                    .transition_mode
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)?;
                autd3_driver::datagram::SwapSegment::GainSTM(
                    autd3_driver::firmware::fpga::Segment::from(Segment::try_from(inner.segment)?),
                    autd3_driver::firmware::fpga::TransitionMode::from_msg(mode)?,
                )
            }
        })
    }
}
