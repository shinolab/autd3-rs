use std::time::Duration;

use crate::{error::*, pb::*, traits::*};

use autd3::derive::AUTDDriverError;
use autd3_driver::datagram::IntoDatagramWithSegment;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[doc(hidden)]
pub struct LightweightServer<
    L: autd3_driver::link::AsyncLinkBuilder + 'static,
    F: Fn() -> L + Send + Sync + 'static,
> where
    L::L: Sync,
{
    autd: RwLock<Option<autd3::Controller<L::L>>>,
    link: F,
}

#[derive(Debug)]
pub struct DatagramWithTimeoutAndParallelThreshold<D: autd3_driver::datagram::Datagram> {
    datagram: D,
    timeout: Option<Duration>,
    parallel_threshold: Option<usize>,
}

impl<D: autd3_driver::datagram::Datagram> autd3_driver::datagram::Datagram
    for DatagramWithTimeoutAndParallelThreshold<D>
{
    type G = D::G;

    fn operation_generator(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::G, AUTDDriverError> {
        self.datagram.operation_generator(geometry)
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        self.timeout.or(self.datagram.timeout())
    }

    fn parallel_threshold(&self) -> Option<usize> {
        self.parallel_threshold
            .or(self.datagram.parallel_threshold())
    }
}

impl<L: autd3_driver::link::AsyncLinkBuilder + 'static, F: Fn() -> L + Send + Sync + 'static>
    LightweightServer<L, F>
where
    L::L: Sync,
{
    pub fn new(f: F) -> Self {
        LightweightServer {
            autd: RwLock::new(None),
            link: f,
        }
    }

    fn parse_gain(gain: &Gain) -> Result<autd3_driver::datagram::BoxedGain, AUTDProtoBufError> {
        use autd3_driver::datagram::IntoBoxedGain;
        Ok(match &gain.gain {
            Some(gain::Gain::Focus(msg)) => autd3::gain::Focus::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Bessel(msg)) => autd3::gain::Bessel::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Plane(msg)) => autd3::gain::Plane::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Uniform(msg)) => autd3::gain::Uniform::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Null(msg)) => autd3::gain::Null::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Lm(msg)) => autd3_gain_holo::LM::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Gs(msg)) => autd3_gain_holo::GS::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Naive(msg)) => autd3_gain_holo::Naive::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Gspat(msg)) => autd3_gain_holo::GSPAT::from_msg(msg)?.into_boxed(),
            Some(gain::Gain::Greedy(msg)) => autd3_gain_holo::Greedy::from_msg(msg)?.into_boxed(),
            None => return Err(AUTDProtoBufError::NotSupportedData),
        })
    }

    fn parse_gain_with_segment(
        gain: &GainWithSegment,
    ) -> Result<
        autd3_driver::datagram::DatagramWithSegment<autd3_driver::datagram::BoxedGain>,
        AUTDProtoBufError,
    > {
        let g = Self::parse_gain(
            gain.gain
                .as_ref()
                .ok_or(AUTDProtoBufError::DataParseError)?,
        )?;
        let segment = autd3_driver::firmware::fpga::Segment::from(Segment::try_from(gain.segment)?);
        let transition_mode = gain
            .transition_mode
            .as_ref()
            .map(autd3_driver::firmware::fpga::TransitionMode::from_msg)
            .transpose()?;
        Ok(g.with_segment(segment, transition_mode))
    }

    fn parse_modulation(
        modulation: &Modulation,
    ) -> Result<autd3_driver::datagram::BoxedModulation, AUTDProtoBufError> {
        use autd3_driver::datagram::IntoBoxedModulation;
        Ok(match &modulation.modulation {
            Some(modulation::Modulation::Static(msg)) => {
                autd3::prelude::Static::from_msg(msg)?.into_boxed()
            }
            Some(modulation::Modulation::SineNearest(msg)) => autd3::prelude::Sine::<
                autd3::modulation::sampling_mode::NearestFreq,
            >::from_msg(msg)?
            .into_boxed(),
            Some(modulation::Modulation::SineExact(msg)) => {
                autd3::prelude::Sine::<autd3::modulation::sampling_mode::ExactFreq>::from_msg(msg)?
                    .into_boxed()
            }
            Some(modulation::Modulation::SineExactFloat(msg)) => autd3::prelude::Sine::<
                autd3::modulation::sampling_mode::ExactFreqFloat,
            >::from_msg(msg)?
            .into_boxed(),
            Some(modulation::Modulation::SquareNearest(msg)) => autd3::prelude::Square::<
                autd3::modulation::sampling_mode::NearestFreq,
            >::from_msg(msg)?
            .into_boxed(),
            Some(modulation::Modulation::SquareExact(msg)) => autd3::prelude::Square::<
                autd3::modulation::sampling_mode::ExactFreq,
            >::from_msg(msg)?
            .into_boxed(),
            Some(modulation::Modulation::SquareExactFloat(msg)) => autd3::prelude::Square::<
                autd3::modulation::sampling_mode::ExactFreqFloat,
            >::from_msg(msg)?
            .into_boxed(),
            None => return Err(AUTDProtoBufError::DataParseError),
        })
    }

    fn parse_modulation_with_segment(
        modulation: &ModulationWithSegment,
    ) -> Result<
        autd3_driver::datagram::DatagramWithSegment<autd3_driver::datagram::BoxedModulation>,
        AUTDProtoBufError,
    > {
        let m = Self::parse_modulation(
            modulation
                .modulation
                .as_ref()
                .ok_or(AUTDProtoBufError::DataParseError)?,
        )?;
        let segment =
            autd3_driver::firmware::fpga::Segment::from(Segment::try_from(modulation.segment)?);
        let transition_mode = modulation
            .transition_mode
            .as_ref()
            .map(autd3_driver::firmware::fpga::TransitionMode::from_msg)
            .transpose()?;
        Ok(m.with_segment(segment, transition_mode))
    }
}

#[tonic::async_trait]
impl<L: autd3_driver::link::AsyncLinkBuilder + 'static, F: Fn() -> L + Send + Sync + 'static>
    ecat_light_server::EcatLight for LightweightServer<L, F>
where
    L::L: Sync,
{
    async fn open(
        &self,
        req: Request<OpenRequestLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.take() {
            if let Err(e) = autd.close().await {
                return Ok(Response::new(SendResponseLightweight {
                    success: false,
                    err: true,
                    msg: format!("{}", e),
                }));
            }
        }
        let req = req.into_inner();
        if let Some(ref geometry) = req.geometry {
            if let Ok(geometry) = autd3_driver::geometry::Geometry::from_msg(geometry) {
                #[allow(unused_mut)]
                let mut builder = autd3::Controller::builder(geometry.iter().map(|d| {
                    autd3::prelude::AUTD3::new(*d[0].position()).with_rotation(*d.rotation())
                }))
                .with_default_parallel_threshold(geometry.default_parallel_threshold() as _)
                .with_default_timeout(std::time::Duration::from_nanos(req.default_timeout))
                .with_send_interval(Duration::from_nanos(req.send_interval))
                .with_receive_interval(Duration::from_nanos(req.receive_interval));
                *self.autd.write().await = match builder.open((self.link)()).await {
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
        } else {
            Ok(Response::new(SendResponseLightweight {
                success: false,
                err: true,
                msg: "Geometry is not configured".to_string(),
            }))
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
                            cpu_major_version: f.cpu().major().0 as _,
                            cpu_minor_version: f.cpu().minor().0 as _,
                            fpga_major_version: f.fpga().major().0 as _,
                            fpga_minor_version: f.fpga().minor().0 as _,
                            fpga_function_bits: f.fpga().function_bits() as _,
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
        req: Request<Datagram>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            let datagram = req.into_inner();
            let parallel_threshold = datagram.parallel_threshold.map(|v| v as usize);
            let timeout = datagram.timeout.map(Duration::from_nanos);
            let res = match datagram.datagram {
                Some(datagram::Datagram::Gain(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: Self::parse_gain(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::GainWithSegment(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: Self::parse_gain_with_segment(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::Modulation(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: Self::parse_modulation(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::ModulationWithSegment(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: Self::parse_modulation_with_segment(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::Clear(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::Clear::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::Silencer(ref msg)) => match msg.config {
                    Some(silencer::Config::FixedUpdateRate(ref msg)) => {
                        autd.send(DatagramWithTimeoutAndParallelThreshold {
                            datagram: autd3_driver::datagram::Silencer::<
                                autd3_driver::datagram::FixedUpdateRate,
                            >::from_msg(msg)?,
                            timeout,
                            parallel_threshold,
                        })
                        .await
                    }
                    Some(silencer::Config::FixedCompletionTime(ref msg)) => {
                        autd.send(DatagramWithTimeoutAndParallelThreshold {
                            datagram: autd3_driver::datagram::Silencer::<
                                autd3_driver::datagram::FixedCompletionTime,
                            >::from_msg(msg)?,
                            timeout,
                            parallel_threshold,
                        })
                        .await
                    }
                    None => return Err(AUTDProtoBufError::NotSupportedData.into()),
                },
                Some(datagram::Datagram::Synchronize(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::Synchronize::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::ForceFan(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::ForceFan::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::ReadsFpgaState(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::ReadsFPGAState::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::GainStm(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::GainSTM::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::GainStmWithSegment(ref msg)) => {
                    let stm = autd3_driver::datagram::GainSTM::from_msg(
                        msg.gain_stm
                            .as_ref()
                            .ok_or(AUTDProtoBufError::DataParseError)?,
                    )?;
                    let segment = autd3_driver::firmware::fpga::Segment::from(
                        Segment::try_from(msg.segment)
                            .map_err(|_| AUTDProtoBufError::DataParseError)?,
                    );
                    let transition_mode = match msg.transition_mode.as_ref() {
                        Some(mode) => Some(autd3_driver::firmware::fpga::TransitionMode::from_msg(
                            mode,
                        )?),
                        None => None,
                    };
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: stm.with_segment(segment, transition_mode),
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                Some(datagram::Datagram::FociStm(ref msg)) => seq_macro::seq!(K in 1..=8 {
                    match msg.inner{
                    #(
                        Some(foci_stm::Inner::N~K(ref msg)) => {
                            autd.send(DatagramWithTimeoutAndParallelThreshold {
                                datagram: autd3_driver::datagram::FociSTM::from_msg(msg)?,
                                timeout,
                                parallel_threshold
                            }).await
                        },
                    )*
                    None => return Err(AUTDProtoBufError::NotSupportedData.into()),
                }}),
                Some(datagram::Datagram::FociStmWithSegment(ref msg)) => {
                    let inner = msg
                        .foci_stm
                        .as_ref()
                        .ok_or(AUTDProtoBufError::DataParseError)?;
                    let segment = autd3_driver::firmware::fpga::Segment::from(
                        Segment::try_from(msg.segment)
                            .map_err(|_| AUTDProtoBufError::DataParseError)?,
                    );
                    let transition_mode = msg
                        .transition_mode
                        .as_ref()
                        .map(autd3_driver::firmware::fpga::TransitionMode::from_msg)
                        .transpose()?;
                    seq_macro::seq!(K in 1..=8 {
                        match inner.inner {
                        #(
                            Some(foci_stm::Inner::N~K(ref msg)) => {
                                autd.send(DatagramWithTimeoutAndParallelThreshold {
                                    datagram: autd3_driver::datagram::FociSTM::from_msg(msg)?.with_segment(segment, transition_mode),
                                    timeout,
                                    parallel_threshold
                                }).await
                            },
                        )*
                        None => return Err(AUTDProtoBufError::NotSupportedData.into()),
                    }})
                }
                Some(datagram::Datagram::SwapSegment(ref msg)) => {
                    autd.send(DatagramWithTimeoutAndParallelThreshold {
                        datagram: autd3_driver::datagram::SwapSegment::from_msg(msg)?,
                        timeout,
                        parallel_threshold,
                    })
                    .await
                }
                None => return Err(AUTDProtoBufError::NotSupportedData.into()),
            };
            match res {
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
                msg: "Geometry is not configured".to_string(),
            }))
        }
    }

    async fn close(
        &self,
        _: Request<CloseRequestLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.take() {
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
