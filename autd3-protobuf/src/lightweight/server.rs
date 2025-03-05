use crate::{error::*, pb::*, traits::*};

use autd3_core::{defined::Freq, link::LinkError};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

#[doc(hidden)]
pub struct LightweightServer<
    L: autd3_core::link::AsyncLink + 'static,
    F: Fn() -> Result<L, LinkError> + Send + Sync + 'static,
> where
    L: Sync,
{
    autd: RwLock<Option<autd3::r#async::Controller<L>>>,
    link: F,
}

impl<
    L: autd3_core::link::AsyncLink + 'static,
    F: Fn() -> Result<L, LinkError> + Send + Sync + 'static,
> LightweightServer<L, F>
where
    L: Sync,
{
    #[must_use]
    pub fn new(f: F) -> Self {
        LightweightServer {
            autd: RwLock::new(None),
            link: f,
        }
    }

    async fn send_datagram(
        autd: &mut autd3::r#async::Controller<L>,
        datagram: Datagram,
    ) -> Result<(), AUTDProtoBufError> {
        use autd3_driver::datagram::*;
        let datagram = datagram.datagram.ok_or(AUTDProtoBufError::DataParseError)?;
        match datagram {
            datagram::Datagram::Clear(msg) => autd.send(Clear::from_msg(msg)?).await?,
            datagram::Datagram::Synchronize(msg) => autd.send(Synchronize::from_msg(msg)?).await?,
            datagram::Datagram::ForceFan(msg) => autd.send(ForceFan::from_msg(msg)?).await?,
            datagram::Datagram::ReadsFpgaState(msg) => {
                autd.send(ReadsFPGAState::from_msg(msg)?).await?
            }
            datagram::Datagram::Silencer(msg) => {
                use autd3::driver::datagram::*;
                use silencer::Config;
                let config = msg.config.ok_or(AUTDProtoBufError::DataParseError)?;
                let target = autd3::driver::firmware::fpga::SilencerTarget::from_msg(msg.target)?;
                match config {
                    Config::FixedUpdateRate(msg) => {
                        autd.send(Silencer {
                            config: FixedUpdateRate::from_msg(msg)?,
                            target,
                        })
                        .await?
                    }
                    Config::FixedCompletionTime(msg) => {
                        autd.send(Silencer {
                            config: FixedCompletionTime::from_msg(msg)?,
                            target,
                        })
                        .await?
                    }
                    Config::FixedCompletionSteps(msg) => {
                        autd.send(Silencer {
                            config: FixedCompletionSteps::from_msg(msg)?,
                            target,
                        })
                        .await?
                    }
                }
            }
            datagram::Datagram::SwapSegment(msg) => autd.send(SwapSegment::from_msg(msg)?).await?,
            datagram::Datagram::Modulation(msg) => {
                use crate::pb::modulation::Modulation;
                use autd3::modulation::*;
                let modulation = msg.modulation.ok_or(AUTDProtoBufError::DataParseError)?;
                match modulation {
                    Modulation::Static(msg) => autd.send(Static::from_msg(msg)?).await?,
                    Modulation::SineNearest(msg) => {
                        autd.send(Sine::<sampling_mode::Nearest>::from_msg(msg)?)
                            .await?
                    }
                    Modulation::SineExact(msg) => {
                        autd.send(Sine::<Freq<u32>>::from_msg(msg)?).await?
                    }
                    Modulation::SineExactFloat(msg) => {
                        autd.send(Sine::<Freq<f32>>::from_msg(msg)?).await?
                    }
                    Modulation::SquareNearest(msg) => {
                        autd.send(Square::<sampling_mode::Nearest>::from_msg(msg)?)
                            .await?
                    }
                    Modulation::SquareExact(msg) => {
                        autd.send(Square::<Freq<u32>>::from_msg(msg)?).await?
                    }
                    Modulation::SquareExactFloat(msg) => {
                        autd.send(Square::<Freq<f32>>::from_msg(msg)?).await?
                    }
                }
            }
            datagram::Datagram::Gain(msg) => {
                use crate::pb::gain::Gain;
                use autd3::gain::*;
                use autd3_gain_holo::*;
                let gain = msg.gain.ok_or(AUTDProtoBufError::DataParseError)?;
                match gain {
                    Gain::Focus(msg) => autd.send(Focus::from_msg(msg)?).await?,
                    Gain::Bessel(msg) => autd.send(Bessel::from_msg(msg)?).await?,
                    Gain::Plane(msg) => autd.send(Plane::from_msg(msg)?).await?,
                    Gain::Uniform(msg) => autd.send(Uniform::from_msg(msg)?).await?,
                    Gain::Null(msg) => autd.send(Null::from_msg(msg)?).await?,
                    Gain::Lm(msg) => autd.send(LM::from_msg(msg)?).await?,
                    Gain::Gs(msg) => autd.send(GS::from_msg(msg)?).await?,
                    Gain::Naive(msg) => autd.send(Naive::from_msg(msg)?).await?,
                    Gain::Gspat(msg) => autd.send(GSPAT::from_msg(msg)?).await?,
                    Gain::Greedy(msg) => autd.send(Greedy::from_msg(msg)?).await?,
                }
            }
            datagram::Datagram::FociStm(msg) => {
                if msg.foci.is_empty() {
                    return Err(AUTDProtoBufError::DataParseError);
                }
                match msg.foci[0].points.len() {
                    1 => autd.send(FociSTM::<1, _, _>::from_msg(msg)?).await?,
                    2 => autd.send(FociSTM::<2, _, _>::from_msg(msg)?).await?,
                    3 => autd.send(FociSTM::<3, _, _>::from_msg(msg)?).await?,
                    4 => autd.send(FociSTM::<4, _, _>::from_msg(msg)?).await?,
                    5 => autd.send(FociSTM::<5, _, _>::from_msg(msg)?).await?,
                    6 => autd.send(FociSTM::<6, _, _>::from_msg(msg)?).await?,
                    7 => autd.send(FociSTM::<7, _, _>::from_msg(msg)?).await?,
                    8 => autd.send(FociSTM::<8, _, _>::from_msg(msg)?).await?,
                    _ => return Err(AUTDProtoBufError::DataParseError),
                }
            }
            datagram::Datagram::GainStm(msg) => {
                autd.send(autd3_driver::datagram::GainSTM::from_msg(msg)?)
                    .await?
            }
            datagram::Datagram::WithSegment(msg) => {
                let segment = autd3::driver::firmware::fpga::Segment::from_msg(msg.segment)?;
                let transition_mode = msg
                    .transition_mode
                    .map(autd3::driver::firmware::fpga::TransitionMode::from_msg)
                    .transpose()?;
                let inner = msg.inner.ok_or(AUTDProtoBufError::DataParseError)?;
                match inner {
                    with_segment::Inner::Gain(msg) => {
                        use crate::pb::gain::Gain;
                        use autd3::gain::*;
                        use autd3_gain_holo::*;
                        let gain = msg.gain.ok_or(AUTDProtoBufError::DataParseError)?;
                        match gain {
                            Gain::Focus(msg) => {
                                autd.send(WithSegment {
                                    inner: Focus::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Bessel(msg) => {
                                autd.send(WithSegment {
                                    inner: Bessel::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Plane(msg) => {
                                autd.send(WithSegment {
                                    inner: Plane::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Uniform(msg) => {
                                autd.send(WithSegment {
                                    inner: Uniform::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Null(msg) => {
                                autd.send(WithSegment {
                                    inner: Null::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Lm(msg) => {
                                autd.send(WithSegment {
                                    inner: LM::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Gs(msg) => {
                                autd.send(WithSegment {
                                    inner: GS::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Naive(msg) => {
                                autd.send(WithSegment {
                                    inner: Naive::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Gspat(msg) => {
                                autd.send(WithSegment {
                                    inner: GSPAT::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Gain::Greedy(msg) => {
                                autd.send(WithSegment {
                                    inner: Greedy::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                        }
                    }
                    with_segment::Inner::Modulation(msg) => {
                        use crate::pb::modulation::Modulation;
                        use autd3::modulation::*;
                        let modulation = msg.modulation.ok_or(AUTDProtoBufError::DataParseError)?;
                        match modulation {
                            Modulation::Static(msg) => {
                                autd.send(WithSegment {
                                    inner: Static::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineNearest(msg) => {
                                autd.send(WithSegment {
                                    inner: Sine::<sampling_mode::Nearest>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineExact(msg) => {
                                autd.send(WithSegment {
                                    inner: Sine::<Freq<u32>>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineExactFloat(msg) => {
                                autd.send(WithSegment {
                                    inner: Sine::<Freq<f32>>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareNearest(msg) => {
                                autd.send(WithSegment {
                                    inner: Square::<sampling_mode::Nearest>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareExact(msg) => {
                                autd.send(WithSegment {
                                    inner: Square::<Freq<u32>>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareExactFloat(msg) => {
                                autd.send(WithSegment {
                                    inner: Square::<Freq<f32>>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                        }
                    }
                    with_segment::Inner::FociStm(msg) => {
                        if msg.foci.is_empty() {
                            return Err(AUTDProtoBufError::DataParseError);
                        }
                        match msg.foci[0].points.len() {
                            1 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<1, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            2 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<2, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            3 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<3, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            4 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<4, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            5 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<5, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            6 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<6, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            7 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<7, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            8 => {
                                autd.send(WithSegment {
                                    inner: FociSTM::<8, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            _ => return Err(AUTDProtoBufError::DataParseError),
                        }
                    }
                    with_segment::Inner::GainStm(msg) => {
                        autd.send(WithSegment {
                            inner: autd3_driver::datagram::GainSTM::from_msg(msg)?,
                            segment,
                            transition_mode,
                        })
                        .await?
                    }
                }
            }
            datagram::Datagram::WithLoopBehavior(msg) => {
                let segment = autd3::driver::firmware::fpga::Segment::from_msg(msg.segment)?;
                let transition_mode = msg
                    .transition_mode
                    .map(autd3::driver::firmware::fpga::TransitionMode::from_msg)
                    .transpose()?;
                let loop_behavior = autd3::driver::firmware::fpga::LoopBehavior::from_msg(
                    msg.loop_behavior.ok_or(AUTDProtoBufError::DataParseError)?,
                )?;
                let inner = msg.inner.ok_or(AUTDProtoBufError::DataParseError)?;
                match inner {
                    with_loop_behavior::Inner::Modulation(msg) => {
                        use crate::pb::modulation::Modulation;
                        use autd3::modulation::*;
                        let modulation = msg.modulation.ok_or(AUTDProtoBufError::DataParseError)?;
                        match modulation {
                            Modulation::Static(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Static::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineNearest(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Sine::<sampling_mode::Nearest>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineExact(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Sine::<Freq<u32>>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SineExactFloat(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Sine::<Freq<f32>>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareNearest(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Square::<sampling_mode::Nearest>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareExact(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Square::<Freq<u32>>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            Modulation::SquareExactFloat(msg) => {
                                autd.send(WithLoopBehavior {
                                    inner: Square::<Freq<f32>>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                        }
                    }
                    with_loop_behavior::Inner::FociStm(msg) => {
                        if msg.foci.is_empty() {
                            return Err(AUTDProtoBufError::DataParseError);
                        }
                        match msg.foci[0].points.len() {
                            1 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<1, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            2 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<2, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            3 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<3, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            4 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<4, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            5 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<5, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            6 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<6, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            7 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<7, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            8 => {
                                autd.send(WithLoopBehavior {
                                    inner: FociSTM::<8, _, _>::from_msg(msg)?,
                                    loop_behavior,
                                    segment,
                                    transition_mode,
                                })
                                .await?
                            }
                            _ => return Err(AUTDProtoBufError::DataParseError),
                        }
                    }
                    with_loop_behavior::Inner::GainStm(msg) => {
                        autd.send(WithLoopBehavior {
                            inner: autd3_driver::datagram::GainSTM::from_msg(msg)?,
                            loop_behavior,
                            segment,
                            transition_mode,
                        })
                        .await?
                    }
                }
            }
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl<
    L: autd3_core::link::AsyncLink + 'static,
    F: Fn() -> Result<L, LinkError> + Send + Sync + 'static,
> ecat_light_server::EcatLight for LightweightServer<L, F>
where
    L: Sync,
{
    async fn open(
        &self,
        req: Request<OpenRequestLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.take() {
            if let Err(e) = autd.close().await {
                return Ok(Response::new(SendResponseLightweight {
                    err: true,
                    msg: format!("{}", e),
                }));
            }
        }
        let req = req.into_inner();
        if let Some(geometry) = req.geometry {
            if let Ok(geometry) = autd3_core::geometry::Geometry::from_msg(geometry) {
                *self.autd.write().await = match autd3::r#async::Controller::open(
                    geometry.iter().map(|d| autd3::prelude::AUTD3 {
                        pos: *d[0].position(),
                        rot: *d.rotation(),
                    }),
                    match (self.link)() {
                        Ok(link) => link,
                        Err(e) => {
                            return Ok(Response::new(SendResponseLightweight {
                                err: true,
                                msg: format!("Failed to open link: {}", e),
                            }));
                        }
                    },
                )
                .await
                {
                    Ok(autd) => Some(autd),
                    Err(e) => {
                        return Ok(Response::new(SendResponseLightweight {
                            err: true,
                            msg: format!("{}", e),
                        }));
                    }
                };
                Ok(Response::new(SendResponseLightweight {
                    err: false,
                    msg: String::new(),
                }))
            } else {
                return Ok(Response::new(SendResponseLightweight {
                    err: true,
                    msg: "Failed to parse Geometry".to_string(),
                }));
            }
        } else {
            Ok(Response::new(SendResponseLightweight {
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
                    err: false,
                    msg: String::new(),
                    firmware_version_list: list
                        .iter()
                        .map(|f| firmware_version_response_lightweight::FirmwareVersion {
                            cpu_major_version: f.cpu.major.0 as _,
                            cpu_minor_version: f.cpu.minor.0 as _,
                            fpga_major_version: f.fpga.major.0 as _,
                            fpga_minor_version: f.fpga.minor.0 as _,
                            fpga_function_bits: f.fpga.function_bits as _,
                        })
                        .collect(),
                })),
                Err(e) => {
                    return Ok(Response::new(FirmwareVersionResponseLightweight {
                        err: true,
                        msg: format!("{}", e),
                        firmware_version_list: Vec::new(),
                    }));
                }
            }
        } else {
            Ok(Response::new(FirmwareVersionResponseLightweight {
                err: true,
                msg: "Geometry is not configured".to_string(),
                firmware_version_list: Vec::new(),
            }))
        }
    }

    async fn fpga_state(
        &self,
        _req: Request<FpgaStateRequestLightweight>,
    ) -> Result<Response<FpgaStateResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            match autd.fpga_state().await {
                Ok(list) => Ok(Response::new(FpgaStateResponseLightweight {
                    err: false,
                    msg: String::new(),
                    fpga_state_list: list
                        .iter()
                        .map(|f| fpga_state_response_lightweight::FpgaState {
                            state: f.map(|s| s.state() as _),
                        })
                        .collect(),
                })),
                Err(e) => {
                    return Ok(Response::new(FpgaStateResponseLightweight {
                        err: true,
                        msg: format!("{}", e),
                        fpga_state_list: Vec::new(),
                    }));
                }
            }
        } else {
            Ok(Response::new(FpgaStateResponseLightweight {
                err: true,
                msg: "Geometry is not configured".to_string(),
                fpga_state_list: Vec::new(),
            }))
        }
    }

    async fn send(
        &self,
        req: Request<Datagram>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            let datagram = req.into_inner();
            match Self::send_datagram(autd, datagram).await {
                Ok(_) => Ok(Response::new(SendResponseLightweight {
                    err: false,
                    msg: String::new(),
                })),
                Err(e) => Ok(Response::new(SendResponseLightweight {
                    err: true,
                    msg: format!("{}", e),
                })),
            }
        } else {
            Ok(Response::new(SendResponseLightweight {
                err: true,
                msg: "Controller is not opened".to_string(),
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
                    err: false,
                    msg: String::new(),
                })),
                Err(e) => Ok(Response::new(SendResponseLightweight {
                    err: true,
                    msg: format!("{}", e),
                })),
            }
        } else {
            Ok(Response::new(SendResponseLightweight {
                err: true,
                msg: "Controller is not opened".to_string(),
            }))
        }
    }
}
