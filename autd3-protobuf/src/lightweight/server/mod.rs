mod gain;
mod modulation;
mod null;
mod tuple;

use std::{num::NonZeroU32, time::Duration};

use crate::{error::*, pb::*, traits::*};

use autd3::datagram::IntoBoxedDatagram;
use autd3_core::link::LinkError;
use autd3_driver::datagram::BoxedDatagram;
use gain::gain_into_boxed;
use modulation::modulation_into_boxed;
use null::NullDatagram;
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
}

fn into_boxed_datagram(datagram: datagram::Datagram) -> Result<BoxedDatagram, AUTDProtoBufError> {
    use autd3_driver::datagram::*;
    match datagram {
        datagram::Datagram::Clear(msg) => Clear::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
        datagram::Datagram::Synchronize(msg) => {
            Synchronize::from_msg(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::ForceFan(msg) => {
            ForceFan::from_msg(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::ReadsFpgaState(msg) => {
            ReadsFPGAState::from_msg(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::Silencer(msg) => {
            use autd3::driver::datagram::*;
            use silencer::Config;
            let config = msg.config.ok_or(AUTDProtoBufError::DataParseError)?;
            let target = autd3::driver::firmware::fpga::SilencerTarget::from_msg(msg.target)?;
            Ok(match config {
                Config::FixedUpdateRate(msg) => Silencer {
                    config: FixedUpdateRate::from_msg(msg)?,
                    target,
                }
                .into_boxed(),
                Config::FixedCompletionTime(msg) => Silencer {
                    config: FixedCompletionTime::from_msg(msg)?,
                    target,
                }
                .into_boxed(),
                Config::FixedCompletionSteps(msg) => Silencer {
                    config: FixedCompletionSteps::from_msg(msg)?,
                    target,
                }
                .into_boxed(),
            })
        }
        datagram::Datagram::SwapSegment(msg) => {
            SwapSegment::from_msg(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::Modulation(msg) => {
            modulation_into_boxed(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::Gain(msg) => gain_into_boxed(msg).map(IntoBoxedDatagram::into_boxed),
        datagram::Datagram::FociStm(msg) => {
            if msg.foci.is_empty() {
                return Err(AUTDProtoBufError::DataParseError);
            }
            match msg.foci[0].points.len() {
                1 => FociSTM::<1, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                2 => FociSTM::<2, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                3 => FociSTM::<3, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                4 => FociSTM::<4, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                5 => FociSTM::<5, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                6 => FociSTM::<6, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                7 => FociSTM::<7, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                8 => FociSTM::<8, _, _>::from_msg(msg).map(IntoBoxedDatagram::into_boxed),
                _ => Err(AUTDProtoBufError::DataParseError),
            }
        }
        datagram::Datagram::GainStm(msg) => {
            autd3_driver::datagram::GainSTM::from_msg(msg).map(IntoBoxedDatagram::into_boxed)
        }
        datagram::Datagram::WithSegment(msg) => {
            let segment = autd3::driver::firmware::fpga::Segment::from_msg(msg.segment)?;
            let transition_mode = msg
                .transition_mode
                .map(autd3::driver::firmware::fpga::TransitionMode::from_msg)
                .transpose()?;
            let inner = msg.inner.ok_or(AUTDProtoBufError::DataParseError)?;
            match inner {
                with_segment::Inner::Gain(msg) => gain_into_boxed(msg).map(|gain| {
                    WithSegment {
                        inner: gain,
                        segment,
                        transition_mode,
                    }
                    .into_boxed()
                }),
                with_segment::Inner::Modulation(msg) => modulation_into_boxed(msg).map(|m| {
                    WithSegment {
                        inner: m,
                        segment,
                        transition_mode,
                    }
                    .into_boxed()
                }),
                with_segment::Inner::FociStm(msg) => {
                    if msg.foci.is_empty() {
                        return Err(AUTDProtoBufError::DataParseError);
                    }
                    Ok(match msg.foci[0].points.len() {
                        1 => WithSegment {
                            inner: FociSTM::<1, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        2 => WithSegment {
                            inner: FociSTM::<2, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        3 => WithSegment {
                            inner: FociSTM::<3, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        4 => WithSegment {
                            inner: FociSTM::<4, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        5 => WithSegment {
                            inner: FociSTM::<5, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        6 => WithSegment {
                            inner: FociSTM::<6, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        7 => WithSegment {
                            inner: FociSTM::<7, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        8 => WithSegment {
                            inner: FociSTM::<8, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                        }
                        .into_boxed(),
                        _ => return Err(AUTDProtoBufError::DataParseError),
                    })
                }
                with_segment::Inner::GainStm(msg) => autd3_driver::datagram::GainSTM::from_msg(msg)
                    .map(|stm| {
                        WithSegment {
                            inner: stm,
                            segment,
                            transition_mode,
                        }
                        .into_boxed()
                    }),
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
                with_loop_behavior::Inner::Modulation(msg) => modulation_into_boxed(msg).map(|m| {
                    WithLoopBehavior {
                        inner: m,
                        segment,
                        transition_mode,
                        loop_behavior,
                    }
                    .into_boxed()
                }),
                with_loop_behavior::Inner::FociStm(msg) => {
                    if msg.foci.is_empty() {
                        return Err(AUTDProtoBufError::DataParseError);
                    }
                    Ok(match msg.foci[0].points.len() {
                        1 => WithLoopBehavior {
                            inner: FociSTM::<1, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        2 => WithLoopBehavior {
                            inner: FociSTM::<2, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        3 => WithLoopBehavior {
                            inner: FociSTM::<3, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        4 => WithLoopBehavior {
                            inner: FociSTM::<4, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        5 => WithLoopBehavior {
                            inner: FociSTM::<5, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        6 => WithLoopBehavior {
                            inner: FociSTM::<6, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        7 => WithLoopBehavior {
                            inner: FociSTM::<7, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        8 => WithLoopBehavior {
                            inner: FociSTM::<8, _, _>::from_msg(msg)?,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed(),
                        _ => return Err(AUTDProtoBufError::DataParseError),
                    })
                }
                with_loop_behavior::Inner::GainStm(msg) => {
                    autd3_driver::datagram::GainSTM::from_msg(msg).map(|stm| {
                        WithLoopBehavior {
                            inner: stm,
                            segment,
                            transition_mode,
                            loop_behavior,
                        }
                        .into_boxed()
                    })
                }
            }
        }
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
        req: Request<SendRequestLightweight>,
    ) -> Result<Response<SendResponseLightweight>, Status> {
        if let Some(autd) = self.autd.write().await.as_mut() {
            let req = req.into_inner();
            let option = req.sender_option;
            let datagram = req.datagram.ok_or(AUTDProtoBufError::DataParseError)?;
            let d1 = datagram.first.ok_or(AUTDProtoBufError::DataParseError)?;
            let d1 = into_boxed_datagram(d1.datagram.ok_or(AUTDProtoBufError::DataParseError)?)?;
            let d2 = if let Some(d2) = datagram.second {
                into_boxed_datagram(d2.datagram.ok_or(AUTDProtoBufError::DataParseError)?)?
            } else {
                NullDatagram.into_boxed()
            };
            let res =
                match option {
                    Some(option) => {
                        let send_interval = Duration::from_nanos(option.send_interval_ns);
                        let receive_interval = Duration::from_nanos(option.receive_interval_ns);
                        let timeout = option.timeout_ns.map(Duration::from_nanos);
                        let parallel = autd3::controller::ParallelMode::from_msg(option.parallel)?;
                        match option.sleeper.ok_or(AUTDProtoBufError::DataParseError)? {
                            sender_option::Sleeper::Std(std_sleeper) => {
                                autd.sender(autd3::controller::SenderOption::<
                                    autd3::controller::StdSleeper,
                                > {
                                    send_interval,
                                    receive_interval,
                                    timeout,
                                    parallel,
                                    sleeper: autd3::controller::StdSleeper {
                                        timer_resolution: std_sleeper
                                            .timer_resolution
                                            .and_then(NonZeroU32::new),
                                    },
                                })
                                .send(tuple::BoxedDatagramTuple { d1, d2 })
                                .await
                            }
                            sender_option::Sleeper::Spin(spin_sleeper) => {
                                autd.sender(autd3::controller::SenderOption::<
                                    autd3::controller::SpinSleeper,
                                > {
                                    send_interval,
                                    receive_interval,
                                    timeout,
                                    parallel,
                                    sleeper: autd3::controller::SpinSleeper::new(
                                        spin_sleeper.native_accuracy_ns,
                                    )
                                    .with_spin_strategy(autd3::controller::SpinStrategy::from_msg(
                                        spin_sleeper.spin_strategy,
                                    )?),
                                })
                                .send(tuple::BoxedDatagramTuple { d1, d2 })
                                .await
                            }
                            #[cfg(target_os = "windows")]
                            sender_option::Sleeper::Waitable(_) => {
                                autd.sender(autd3::controller::SenderOption::<
                                    autd3::controller::WaitableSleeper,
                                > {
                                    send_interval,
                                    receive_interval,
                                    timeout,
                                    parallel,
                                    sleeper: autd3::controller::WaitableSleeper::new().map_err(
                                        |_| {
                                            AUTDProtoBufError::Status(tonic::Status::unknown(
                                                "WaitableSleeper",
                                            ))
                                        },
                                    )?,
                                })
                                .send(tuple::BoxedDatagramTuple { d1, d2 })
                                .await
                            }
                            #[cfg(not(target_os = "windows"))]
                            sender_option::Sleeper::Waitable(_) => Err(AUTDProtoBufError::Status(
                                tonic::Status::unimplemented("WaitableSleeper"),
                            )),
                            sender_option::Sleeper::Async(async_sleeper) => {
                                autd.sender(autd3::controller::SenderOption::<
                                    autd3::r#async::controller::AsyncSleeper,
                                > {
                                    send_interval,
                                    receive_interval,
                                    timeout,
                                    parallel,
                                    sleeper: autd3::r#async::controller::AsyncSleeper {
                                        timer_resolution: async_sleeper
                                            .timer_resolution
                                            .and_then(NonZeroU32::new),
                                    },
                                })
                                .send(tuple::BoxedDatagramTuple { d1, d2 })
                                .await
                            }
                        }
                    }
                    None => autd.send(tuple::BoxedDatagramTuple { d1, d2 }).await,
                };
            match res {
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
