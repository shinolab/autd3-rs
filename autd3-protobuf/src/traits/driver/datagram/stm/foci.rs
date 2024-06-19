use autd3_driver::derive::SamplingConfig;

use crate::{pb::*, traits::*};

impl ToMessage for autd3_driver::datagram::FociSTM<1> {
    type Message = FociStm1;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<1>>
{
    type Message = FociStm1;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm1> for autd3_driver::datagram::FociSTM<1> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm1) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap()],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<2> {
    type Message = FociStm2;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<2>>
{
    type Message = FociStm2;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm2> for autd3_driver::datagram::FociSTM<2> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm2) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<3> {
    type Message = FociStm3;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<3>>
{
    type Message = FociStm3;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm3> for autd3_driver::datagram::FociSTM<3> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm3) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<4> {
    type Message = FociStm4;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<4>>
{
    type Message = FociStm4;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm4> for autd3_driver::datagram::FociSTM<4> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm4) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[3]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<5> {
    type Message = FociStm5;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<5>>
{
    type Message = FociStm5;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm5> for autd3_driver::datagram::FociSTM<5> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm5) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[3]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[4]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<6> {
    type Message = FociStm6;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<6>>
{
    type Message = FociStm6;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm6> for autd3_driver::datagram::FociSTM<6> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm6) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[3]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[4]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[5]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<7> {
    type Message = FociStm7;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<7>>
{
    type Message = FociStm7;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm7> for autd3_driver::datagram::FociSTM<7> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm7) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[3]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[4]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[5]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[6]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}

impl ToMessage for autd3_driver::datagram::FociSTM<8> {
    type Message = FociStm8;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: Segment::S0 as _,
                transition_mode: Some(0xFF),
                transition_value: Some(0),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl ToMessage
    for autd3_driver::datagram::DatagramWithSegmentTransition<autd3_driver::datagram::FociSTM<8>>
{
    type Message = FociStm8;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            props: Some(FociStmProps {
                config: Some(self.sampling_config().to_msg(None)),
                loop_behavior: Some(self.loop_behavior().to_msg(None)),
                segment: self.segment() as _,
                transition_mode: self.transition_mode().map(|m| m.mode() as _),
                transition_value: self.transition_mode().map(|m| m.value()),
            }),
            foci: self.iter().map(|p| p.to_msg(None)).collect(),
        }
    }
}

impl FromMessage<FociStm8> for autd3_driver::datagram::FociSTM<8> {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &FociStm8) -> Option<Self> {
        Some(
            autd3_driver::datagram::FociSTM::from_sampling_config(
                SamplingConfig::from_msg(msg.props.as_ref().unwrap().config.as_ref().unwrap())
                    .unwrap(),
                msg.foci.iter().map(|f| {
                    (
                        [
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[0]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[1]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[2]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[3]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[4]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[5]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[6]).unwrap(),
                            autd3_driver::defined::ControlPoint::from_msg(&f.points[7]).unwrap(),
                        ],
                        autd3_driver::firmware::fpga::EmitIntensity::from_msg(
                            f.intensity.as_ref().unwrap(),
                        )
                        .unwrap(),
                    )
                }),
            )
            .with_loop_behavior(autd3_driver::firmware::fpga::LoopBehavior::from_msg(
                msg.props.as_ref().unwrap().loop_behavior.as_ref()?,
            )?),
        )
    }
}
