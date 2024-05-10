use crate::{error::*, pb::*, traits::*};

use autd3::error::AUTDError;
use autd3_driver::datagram::{IntoDatagramWithSegment, IntoDatagramWithSegmentTransition};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[doc(hidden)]
pub struct LightweightServer<
    L: autd3_driver::link::LinkBuilder + Sync + 'static,
    F: Fn() -> L + Send + Sync + 'static,
> {
    autd: RwLock<Option<autd3::Controller<L::L>>>,
    link: F,
}

impl<L: autd3_driver::link::LinkBuilder + Sync + 'static, F: Fn() -> L + Send + Sync + 'static>
    LightweightServer<L, F>
{
    pub fn new(f: F) -> Self {
        LightweightServer {
            autd: RwLock::new(None),
            link: f,
        }
    }

    async fn send_modulation(
        autd: &mut autd3::Controller<L::L>,
        modulation: &Modulation,
    ) -> Result<bool, AUTDError> {
        Ok(match &modulation.modulation {
            Some(modulation::Modulation::Static(msg)) => {
                autd.send(
                    autd3::prelude::Static::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(modulation.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            to_transition_mode(
                                modulation.transition_mode,
                                modulation.transition_value,
                            ),
                        ),
                )
                .await?
            }
            Some(modulation::Modulation::SineNearest(msg)) => {
                autd.send(
                    autd3::prelude::Sine::<autd3::modulation::sampling_mode::NearestFreq>::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(modulation.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            to_transition_mode(
                                modulation.transition_mode,
                                modulation.transition_value,
                            ),
                        ),
                )
                .await?
            }
            Some(modulation::Modulation::SineExact(msg)) => {
                autd.send(
                    autd3::prelude::Sine::<autd3::modulation::sampling_mode::ExactFreqFloat>::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(modulation.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            to_transition_mode(
                                modulation.transition_mode,
                                modulation.transition_value,
                            ),
                        ),
                )
                .await?
            }
            Some(modulation::Modulation::SquareNearest(msg)) => {
                autd.send(
                    autd3::prelude::Square::<autd3::modulation::sampling_mode::NearestFreq>::from_msg(
                        msg,
                    )
                    .ok_or(AUTDProtoBufError::DataParseError)?
                    .with_segment(
                        autd3_driver::firmware::fpga::Segment::from(
                            Segment::try_from(modulation.segment)
                                .ok()
                                .ok_or(AUTDProtoBufError::DataParseError)?,
                        ),
                        to_transition_mode(modulation.transition_mode, modulation.transition_value),
                    ),
                )
                .await?
            }
            Some(modulation::Modulation::SquareExact(msg)) => {
                autd.send(
                    autd3::prelude::Square::<autd3::modulation::sampling_mode::ExactFreqFloat>::from_msg(
                        msg,
                    )
                    .ok_or(AUTDProtoBufError::DataParseError)?
                    .with_segment(
                        autd3_driver::firmware::fpga::Segment::from(
                            Segment::try_from(modulation.segment)
                                .ok()
                                .ok_or(AUTDProtoBufError::DataParseError)?,
                        ),
                        to_transition_mode(modulation.transition_mode, modulation.transition_value),
                    ),
                )
                .await?
            }
            None => return Err(AUTDProtoBufError::NotSupportedData.into()),
        })
    }

    async fn send_silencer(
        autd: &mut autd3::Controller<L::L>,
        msg: &Silencer,
    ) -> Result<bool, AUTDError> {
        Ok(match msg.config {
            Some(silencer::Config::FixedUpdateRate(ref msg)) => {
                autd.send(
                    autd3_driver::datagram::SilencerFixedUpdateRate::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(silencer::Config::FixedCompletionSteps(ref msg)) => {
                autd.send(
                    autd3_driver::datagram::SilencerFixedCompletionSteps::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            None => return Err(AUTDProtoBufError::NotSupportedData.into()),
        })
    }

    async fn send_gain(autd: &mut autd3::Controller<L::L>, gain: &Gain) -> Result<bool, AUTDError> {
        Ok(match &gain.gain {
            Some(gain::Gain::Focus(msg)) => {
                autd.send(
                    autd3::prelude::Focus::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Bessel(msg)) => {
                autd.send(
                    autd3::prelude::Bessel::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Null(msg)) => {
                autd.send(
                    autd3::prelude::Null::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Plane(msg)) => {
                autd.send(
                    autd3::prelude::Plane::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Uniform(msg)) => {
                autd.send(
                    autd3::prelude::Uniform::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Sdp(msg)) => {
                autd.send(
                    autd3_gain_holo::SDP::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Naive(msg)) => {
                autd.send(
                    autd3_gain_holo::Naive::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Gs(msg)) => {
                autd.send(
                    autd3_gain_holo::GS::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Gspat(msg)) => {
                autd.send(
                    autd3_gain_holo::GSPAT::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Lm(msg)) => {
                autd.send(
                    autd3_gain_holo::LM::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            Some(gain::Gain::Greedy(msg)) => {
                autd.send(
                    autd3_gain_holo::Greedy::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?
                        .with_segment(
                            autd3_driver::firmware::fpga::Segment::from(
                                Segment::try_from(gain.segment)
                                    .ok()
                                    .ok_or(AUTDProtoBufError::DataParseError)?,
                            ),
                            gain.transition,
                        ),
                )
                .await?
            }
            None => return Err(AUTDProtoBufError::NotSupportedData.into()),
        })
    }
}

#[tonic::async_trait]
impl<L: autd3_driver::link::LinkBuilder + Sync + 'static, F: Fn() -> L + Send + Sync + 'static>
    ecat_light_server::EcatLight for LightweightServer<L, F>
{
    async fn config_geomety(
        &self,
        req: Request<Geometry>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(mut autd) = self.autd.write().await.take() {
            match autd.close().await {
                Ok(_) => {}
                Err(e) => {
                    return Ok(Response::new(SendResponseLightweight {
                        success: false,
                        err: true,
                        msg: format!("{}", e),
                    }))
                }
            }
        }
        if let Some(geometry) = autd3_driver::geometry::Geometry::from_msg(&req.into_inner()) {
            *self.autd.write().await = match geometry
                .iter()
                .fold(autd3::Controller::builder(), |acc, d| {
                    acc.add_device(
                        autd3::prelude::AUTD3::new(*d[0].position())
                            .with_rotation(*d[0].rotation()),
                    )
                })
                .open((self.link)())
                .await
            {
                Ok(autd) => Some(autd),
                Err(e) => {
                    return Ok(Response::new(SendResponseLightweight {
                        success: false,
                        err: true,
                        msg: format!("{}", e),
                    }))
                }
            };
            Ok(Response::new(SendResponseLightweight {
                success: true,
                err: false,
                msg: String::new(),
            }))
        } else {
            return Ok(Response::new(SendResponseLightweight {
                success: false,
                err: true,
                msg: "Failed to parse Geometry".to_string(),
            }));
        }
    }

    async fn firmware_version(
        &self,
        _req: Request<FirmwareVersionRequestLightweight>,
    ) -> Result<Response<FirmwareVersionResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            match autd.firmware_version().await {
                Ok(list) => Ok(Response::new(FirmwareVersionResponseLightweight {
                    success: true,
                    msg: String::new(),
                    firmware_version_list: list
                        .iter()
                        .map(|f| firmware_version_response_lightweight::FirmwareVersion {
                            cpu_major_version: f.cpu_version_number_major() as _,
                            cpu_minor_version: f.cpu_version_number_minor() as _,
                            fpga_major_version: f.fpga_version_number_major() as _,
                            fpga_minor_version: f.fpga_version_number_minor() as _,
                            fpga_function_bits: f.fpga_function_bits() as _,
                        })
                        .collect(),
                })),
                Err(e) => {
                    return Ok(Response::new(FirmwareVersionResponseLightweight {
                        success: false,
                        msg: format!("{}", e),
                        firmware_version_list: Vec::new(),
                    }))
                }
            }
        } else {
            Ok(Response::new(FirmwareVersionResponseLightweight {
                success: false,
                msg: "Geometry is not configured".to_string(),
                firmware_version_list: Vec::new(),
            }))
        }
    }

    async fn send(
        &self,
        req: Request<DatagramLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            match match req.into_inner().datagram {
                Some(datagram_lightweight::Datagram::Silencer(ref msg)) => {
                    Self::send_silencer(autd, msg).await
                }
                Some(datagram_lightweight::Datagram::Gain(ref msg)) => {
                    Self::send_gain(autd, msg).await
                }
                Some(datagram_lightweight::Datagram::Modulation(ref msg)) => {
                    Self::send_modulation(autd, msg).await
                }
                Some(datagram_lightweight::Datagram::Clear(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::Clear::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::Synchronize(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::Synchronize::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::ForceFan(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::ForceFan::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::ReadsFpgaState(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::ReadsFPGAState::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::FocusStm(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::FocusSTM::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?
                            .with_segment(
                                autd3_driver::firmware::fpga::Segment::from(
                                    Segment::try_from(msg.segment)
                                        .ok()
                                        .ok_or(AUTDProtoBufError::DataParseError)?,
                                ),
                                to_transition_mode(msg.transition_mode, msg.transition_value),
                            ),
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::GainStm(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::GainSTM::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?
                            .with_segment(
                                autd3_driver::firmware::fpga::Segment::from(
                                    Segment::try_from(msg.segment)
                                        .ok()
                                        .ok_or(AUTDProtoBufError::DataParseError)?,
                                ),
                                to_transition_mode(msg.transition_mode, msg.transition_value),
                            ),
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::SwapSegmentGain(ref msg)) => {
                    autd
                        .send(
                            autd3_driver::datagram::SwapSegment::<
                                autd3_driver::datagram::segment::Gain,
                            >::from_msg(msg)
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                        )
                        .await
                }
                Some(datagram_lightweight::Datagram::SwapSegmentModulation(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::SwapSegment::<
                            autd3_driver::datagram::segment::Modulation,
                        >::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::SwapSegmentGainStm(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::SwapSegment::<
                            autd3_driver::datagram::segment::GainSTM,
                        >::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                Some(datagram_lightweight::Datagram::SwapSegmentFocusStm(ref msg)) => {
                    autd.send(
                        autd3_driver::datagram::SwapSegment::<
                            autd3_driver::datagram::segment::FocusSTM,
                        >::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                    )
                    .await
                }
                None => return Err(Status::invalid_argument("No datagram")),
            } {
                Ok(res) => Ok(Response::new(SendResponseLightweight {
                    success: res,
                    err: false,
                    msg: String::new(),
                })),
                Err(e) => Ok(Response::new(SendResponseLightweight {
                    success: false,
                    err: true,
                    msg: format!("{}", e),
                })),
            }
        } else {
            Ok(Response::new(SendResponseLightweight {
                success: false,
                err: true,
                msg: "Geometry is not configured".to_string(),
            }))
        }
    }

    async fn close(
        &self,
        _: Request<CloseRequestLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(mut autd) = self.autd.write().await.take() {
            match autd.close().await {
                Ok(_) => Ok(Response::new(SendResponseLightweight {
                    success: true,
                    err: false,
                    msg: String::new(),
                })),
                Err(e) => Ok(Response::new(SendResponseLightweight {
                    success: false,
                    err: true,
                    msg: format!("{}", e),
                })),
            }
        } else {
            Ok(Response::new(SendResponseLightweight {
                success: false,
                err: true,
                msg: "Controller is not opened".to_string(),
            }))
        }
    }
}
