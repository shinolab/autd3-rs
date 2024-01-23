use crate::{error::*, pb::*, traits::*};

use autd3::error::AUTDError;
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
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(modulation::Modulation::Sine(msg)) => {
                autd.send(
                    autd3::prelude::Sine::from_msg(msg).ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(modulation::Modulation::Square(msg)) => {
                autd.send(
                    autd3::prelude::Square::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            None => return Err(AUTDProtoBufError::NotSupportedData.into()),
        })
    }

    async fn send_silencer(
        autd: &mut autd3::Controller<L::L>,
        msg: &ConfigureSilencer,
    ) -> Result<bool, AUTDError> {
        Ok(match msg.config {
            Some(configure_silencer::Config::FixedUpdateRate(ref msg)) => {
                autd.send(
                    autd3_driver::datagram::ConfigureSilencerFixedUpdateRate::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(configure_silencer::Config::FixedCompletionSteps(ref msg)) => {
                autd.send(
                    autd3_driver::datagram::ConfigureSilencerFixedCompletionSteps::from_msg(msg)
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
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Bessel(msg)) => {
                autd.send(
                    autd3::prelude::Bessel::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Null(msg)) => {
                autd.send(
                    autd3::prelude::Null::from_msg(msg).ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Plane(msg)) => {
                autd.send(
                    autd3::prelude::Plane::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Uniform(msg)) => {
                autd.send(
                    autd3::prelude::Uniform::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Sdp(msg)) => {
                autd.send(
                    autd3_gain_holo::SDP::from_msg(msg).ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Naive(msg)) => {
                autd.send(
                    autd3_gain_holo::Naive::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Gs(msg)) => {
                autd.send(
                    autd3_gain_holo::GS::from_msg(msg).ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Gspat(msg)) => {
                autd.send(
                    autd3_gain_holo::GSPAT::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Lm(msg)) => {
                autd.send(
                    autd3_gain_holo::LM::from_msg(msg).ok_or(AUTDProtoBufError::DataParseError)?,
                )
                .await?
            }
            Some(gain::Gain::Greedy(msg)) => {
                autd.send(
                    autd3_gain_holo::Greedy::from_msg(msg)
                        .ok_or(AUTDProtoBufError::DataParseError)?,
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
                .open_with((self.link)())
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

    async fn firmware_info(
        &self,
        _req: Request<FirmwareInfoRequestLightweight>,
    ) -> Result<Response<FirmwareInfoResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            match autd.firmware_infos().await {
                Ok(list) => Ok(Response::new(FirmwareInfoResponseLightweight {
                    success: true,
                    msg: String::new(),
                    firmware_info_list: list
                        .iter()
                        .map(|f| firmware_info_response_lightweight::FirmwareInfo {
                            cpu_major_version: f.cpu_version_number_major() as _,
                            cpu_minor_version: f.cpu_version_number_minor() as _,
                            fpga_major_version: f.fpga_version_number_major() as _,
                            fpga_minor_version: f.fpga_version_number_minor() as _,
                            fpga_function_bits: f.fpga_function_bits() as _,
                        })
                        .collect(),
                })),
                Err(e) => {
                    return Ok(Response::new(FirmwareInfoResponseLightweight {
                        success: false,
                        msg: format!("{}", e),
                        firmware_info_list: Vec::new(),
                    }))
                }
            }
        } else {
            Ok(Response::new(FirmwareInfoResponseLightweight {
                success: false,
                msg: "Geometry is not configured".to_string(),
                firmware_info_list: Vec::new(),
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
