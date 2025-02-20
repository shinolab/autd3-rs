use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::SwapSegment {
    type Message = SwapSegment;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            variant: Some(match self {
                autd3_driver::datagram::SwapSegment::Gain(segment, transition) => {
                    swap_segment::Variant::Gain(swap_segment::Gain {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::Modulation(segment, transition) => {
                    swap_segment::Variant::Modulation(swap_segment::Modulation {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::FociSTM(segment, transition) => {
                    swap_segment::Variant::FociStm(swap_segment::FociStm {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
                autd3_driver::datagram::SwapSegment::GainSTM(segment, transition) => {
                    swap_segment::Variant::GainStm(swap_segment::GainStm {
                        segment: *segment as _,
                        transition_mode: Some(transition.to_msg(None)?),
                    })
                }
            }),
        })
    }
}

impl FromMessage<SwapSegment> for autd3_driver::datagram::SwapSegment {
    fn from_msg(msg: SwapSegment) -> Result<Self, AUTDProtoBufError> {
        Ok(
            match msg.variant.ok_or(AUTDProtoBufError::DataParseError)? {
                swap_segment::Variant::Gain(value) => autd3_driver::datagram::SwapSegment::Gain(
                    autd3_driver::firmware::fpga::Segment::from_msg(value.segment)?,
                    autd3_driver::firmware::fpga::TransitionMode::from_msg(
                        value
                            .transition_mode
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )?,
                ),
                swap_segment::Variant::Modulation(value) => {
                    autd3_driver::datagram::SwapSegment::Modulation(
                        autd3_driver::firmware::fpga::Segment::from_msg(value.segment)?,
                        autd3_driver::firmware::fpga::TransitionMode::from_msg(
                            value
                                .transition_mode
                                .ok_or(AUTDProtoBufError::DataParseError)?,
                        )?,
                    )
                }
                swap_segment::Variant::FociStm(value) => {
                    autd3_driver::datagram::SwapSegment::FociSTM(
                        autd3_driver::firmware::fpga::Segment::from_msg(value.segment)?,
                        autd3_driver::firmware::fpga::TransitionMode::from_msg(
                            value
                                .transition_mode
                                .ok_or(AUTDProtoBufError::DataParseError)?,
                        )?,
                    )
                }
                swap_segment::Variant::GainStm(value) => {
                    autd3_driver::datagram::SwapSegment::GainSTM(
                        autd3_driver::firmware::fpga::Segment::from_msg(value.segment)?,
                        autd3_driver::firmware::fpga::TransitionMode::from_msg(
                            value
                                .transition_mode
                                .ok_or(AUTDProtoBufError::DataParseError)?,
                        )?,
                    )
                }
            },
        )
    }
}
