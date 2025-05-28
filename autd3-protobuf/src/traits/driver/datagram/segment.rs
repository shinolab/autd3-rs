use crate::{
    AUTDProtoBufError,
    pb::*,
    traits::{DatagramLightweight, FromMessage},
};

impl DatagramLightweight for autd3_driver::datagram::SwapSegment {
    fn into_datagram_lightweight(
        self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<RawDatagram, AUTDProtoBufError> {
        Ok(RawDatagram {
            datagram: Some(raw_datagram::Datagram::SwapSegment(SwapSegment {
                variant: Some(match self {
                    autd3_driver::datagram::SwapSegment::Gain(segment, transition) => {
                        swap_segment::Variant::Gain(swap_segment::Gain {
                            segment: segment as _,
                            transition_mode: Some(transition.into()),
                        })
                    }
                    autd3_driver::datagram::SwapSegment::Modulation(segment, transition) => {
                        swap_segment::Variant::Modulation(swap_segment::Modulation {
                            segment: segment as _,
                            transition_mode: Some(transition.into()),
                        })
                    }
                    autd3_driver::datagram::SwapSegment::FociSTM(segment, transition) => {
                        swap_segment::Variant::FociStm(swap_segment::FociStm {
                            segment: segment as _,
                            transition_mode: Some(transition.into()),
                        })
                    }
                    autd3_driver::datagram::SwapSegment::GainSTM(segment, transition) => {
                        swap_segment::Variant::GainStm(swap_segment::GainStm {
                            segment: segment as _,
                            transition_mode: Some(transition.into()),
                        })
                    }
                }),
            })),
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
