use crate::{
    pb::*,
    traits::{to_transition_mode, FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::datagram::SwapSegment {
    type Message = SwapSegment;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            inner: Some(match self {
                autd3_driver::datagram::SwapSegment::Gain(segment) => {
                    swap_segment::Inner::Gain(SwapSegmentGain {
                        segment: *segment as _,
                    })
                }
                autd3_driver::datagram::SwapSegment::Modulation(segment, transition) => {
                    swap_segment::Inner::Modulation(SwapSegmentModulation {
                        segment: *segment as _,
                        transition_mode: transition.mode() as _,
                        transition_value: transition.value(),
                    })
                }
                autd3_driver::datagram::SwapSegment::FociSTM(segment, transition) => {
                    swap_segment::Inner::FociStm(SwapSegmentFociStm {
                        segment: *segment as _,
                        transition_mode: transition.mode() as _,
                        transition_value: transition.value(),
                    })
                }
                autd3_driver::datagram::SwapSegment::GainSTM(segment, transition) => {
                    swap_segment::Inner::GainStm(SwapSegmentGainStm {
                        segment: *segment as _,
                        transition_mode: transition.mode() as _,
                        transition_value: transition.value(),
                    })
                }
                _ => unimplemented!(),
            }),
        }
    }
}

impl FromMessage<SwapSegment> for autd3_driver::datagram::SwapSegment {
    fn from_msg(msg: &SwapSegment) -> Option<Self> {
        msg.inner.as_ref().and_then(|inner| {
            Some(match inner {
                swap_segment::Inner::Gain(inner) => autd3_driver::datagram::SwapSegment::Gain(
                    autd3_driver::firmware::fpga::Segment::from(
                        Segment::try_from(inner.segment).ok()?,
                    ),
                ),
                swap_segment::Inner::Modulation(inner) => {
                    autd3_driver::datagram::SwapSegment::Modulation(
                        autd3_driver::firmware::fpga::Segment::from(
                            Segment::try_from(inner.segment).ok()?,
                        ),
                        to_transition_mode(
                            Some(inner.transition_mode),
                            Some(inner.transition_value),
                        )
                        .unwrap(),
                    )
                }
                swap_segment::Inner::FociStm(inner) => {
                    autd3_driver::datagram::SwapSegment::FociSTM(
                        autd3_driver::firmware::fpga::Segment::from(
                            Segment::try_from(inner.segment).ok()?,
                        ),
                        to_transition_mode(
                            Some(inner.transition_mode),
                            Some(inner.transition_value),
                        )
                        .unwrap(),
                    )
                }
                swap_segment::Inner::GainStm(inner) => {
                    autd3_driver::datagram::SwapSegment::GainSTM(
                        autd3_driver::firmware::fpga::Segment::from(
                            Segment::try_from(inner.segment).ok()?,
                        ),
                        to_transition_mode(
                            Some(inner.transition_mode),
                            Some(inner.transition_value),
                        )
                        .unwrap(),
                    )
                }
            })
        })
    }
}
