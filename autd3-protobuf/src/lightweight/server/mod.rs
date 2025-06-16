mod gain;
mod modulation;
mod null;
mod tuple;

use crate::{error::*, pb::*, traits::*};

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
    F: Fn() -> Result<L, LinkError> + Send + 'static,
> {
    autd: RwLock<Option<autd3::r#async::Controller<L>>>,
    link: F,
}

impl<L: autd3_core::link::AsyncLink + 'static, F: Fn() -> Result<L, LinkError> + Send + 'static>
    LightweightServer<L, F>
{
    #[must_use]
    pub fn new(f: F) -> Self {
        LightweightServer {
            autd: RwLock::new(None),
            link: f,
        }
    }
}

fn into_boxed_datagram(
    datagram: raw_datagram::Datagram,
) -> Result<BoxedDatagram, AUTDProtoBufError> {
    use autd3_driver::datagram::*;
    match datagram {
        raw_datagram::Datagram::Clear(msg) => Clear::from_msg(msg).map(BoxedDatagram::new),
        raw_datagram::Datagram::Synchronize(msg) => {
            Synchronize::from_msg(msg).map(BoxedDatagram::new)
        }
        raw_datagram::Datagram::ForceFan(msg) => ForceFan::from_msg(msg).map(BoxedDatagram::new),
        raw_datagram::Datagram::ReadsFpgaState(msg) => {
            ReadsFPGAState::from_msg(msg).map(BoxedDatagram::new)
        }
        raw_datagram::Datagram::Silencer(msg) => {
            use autd3::driver::datagram::*;
            use silencer::Config;
            let config = msg.config.ok_or(AUTDProtoBufError::DataParseError)?;
            Ok(match config {
                Config::FixedUpdateRate(msg) => BoxedDatagram::new(Silencer {
                    config: FixedUpdateRate::from_msg(msg)?,
                }),
                Config::FixedCompletionTime(msg) => BoxedDatagram::new(Silencer {
                    config: FixedCompletionTime::from_msg(msg)?,
                }),
                Config::FixedCompletionSteps(msg) => BoxedDatagram::new(Silencer {
                    config: FixedCompletionSteps::from_msg(msg)?,
                }),
            })
        }
        raw_datagram::Datagram::SwapSegment(msg) => {
            SwapSegment::from_msg(msg).map(BoxedDatagram::new)
        }
        raw_datagram::Datagram::Modulation(msg) => {
            modulation_into_boxed(msg).map(BoxedDatagram::new)
        }
        raw_datagram::Datagram::Gain(msg) => gain_into_boxed(msg).map(BoxedDatagram::new),
        raw_datagram::Datagram::FociStm(msg) => {
            if msg.foci.is_empty() {
                return Err(AUTDProtoBufError::DataParseError);
            }
            seq_macro::seq!(N in 0..8 {
                match msg.foci[0].points.len() {
                    #(
                        N => FociSTM::<N, _, _>::from_msg(msg).map(BoxedDatagram::new),
                    )*
                    _ => Err(AUTDProtoBufError::DataParseError),
                }
            })
        }
        raw_datagram::Datagram::GainStm(msg) => {
            autd3_driver::datagram::GainSTM::from_msg(msg).map(BoxedDatagram::new)
        }
        raw_datagram::Datagram::WithSegment(msg) => {
            let segment = autd3::driver::firmware::fpga::Segment::from_msg(msg.segment)?;
            let transition_mode = msg
                .transition_mode
                .map(autd3::driver::firmware::fpga::TransitionMode::from_msg)
                .transpose()?;
            let inner = msg.inner.ok_or(AUTDProtoBufError::DataParseError)?;
            match inner {
                with_segment::Inner::Gain(msg) => gain_into_boxed(msg).map(|gain| {
                    BoxedDatagram::new(WithSegment {
                        inner: gain,
                        segment,
                        transition_mode,
                    })
                }),
                with_segment::Inner::Modulation(msg) => modulation_into_boxed(msg).map(|m| {
                    BoxedDatagram::new(WithSegment {
                        inner: m,
                        segment,
                        transition_mode,
                    })
                }),
                with_segment::Inner::FociStm(msg) => {
                    if msg.foci.is_empty() {
                        return Err(AUTDProtoBufError::DataParseError);
                    }
                    Ok(seq_macro::seq!(N in 0..8 {
                        match msg.foci[0].points.len() {
                            #(
                                N => BoxedDatagram::new(WithSegment {
                                    inner: FociSTM::<N, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                }),
                            )*
                            _ => return Err(AUTDProtoBufError::DataParseError),
                        }
                    }))
                }
                with_segment::Inner::GainStm(msg) => autd3_driver::datagram::GainSTM::from_msg(msg)
                    .map(|stm| {
                        BoxedDatagram::new(WithSegment {
                            inner: stm,
                            segment,
                            transition_mode,
                        })
                    }),
            }
        }
        raw_datagram::Datagram::WithLoopBehavior(msg) => {
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
                    BoxedDatagram::new(WithLoopBehavior {
                        inner: m,
                        segment,
                        transition_mode,
                        loop_behavior,
                    })
                }),
                with_loop_behavior::Inner::FociStm(msg) => {
                    if msg.foci.is_empty() {
                        return Err(AUTDProtoBufError::DataParseError);
                    }
                    Ok(seq_macro::seq!(N in 0..8 {
                        match msg.foci[0].points.len() {
                            #(
                                N => BoxedDatagram::new(WithLoopBehavior {
                                    inner: FociSTM::<N, _, _>::from_msg(msg)?,
                                    segment,
                                    transition_mode,
                                                                loop_behavior,
                                }),
                            )*
                            _ => return Err(AUTDProtoBufError::DataParseError),
                        }
                    }))
                }
                with_loop_behavior::Inner::GainStm(msg) => {
                    autd3_driver::datagram::GainSTM::from_msg(msg).map(|stm| {
                        BoxedDatagram::new(WithLoopBehavior {
                            inner: stm,
                            segment,
                            transition_mode,
                            loop_behavior,
                        })
                    })
                }
            }
        }
    }
}

fn into_datagram_tuple(
    tuple: DatagramTuple,
) -> Result<tuple::BoxedDatagramTuple, AUTDProtoBufError> {
    let d1 = tuple.first.ok_or(AUTDProtoBufError::DataParseError)?;
    let d1 = into_boxed_datagram(d1.datagram.ok_or(AUTDProtoBufError::DataParseError)?)?;
    let d2 = if let Some(d2) = tuple.second {
        into_boxed_datagram(d2.datagram.ok_or(AUTDProtoBufError::DataParseError)?)?
    } else {
        BoxedDatagram::new(NullDatagram)
    };
    Ok(tuple::BoxedDatagramTuple { d1, d2 })
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
            let sleeper = req.sleeper;
            let datagram = req.datagram.ok_or(AUTDProtoBufError::DataParseError)?;

            let (option, sleeper) = match option {
                Some(option) => {
                    let option = autd3::controller::SenderOption::from_msg(option)?;
                    let sleeper = sleeper
                        .map(Box::<dyn autd3::r#async::controller::Sleep>::from_msg)
                        .transpose()?
                        .unwrap_or_else(|| Box::new(autd3::r#async::AsyncSleeper));
                    (option, sleeper)
                }
                None => (
                    autd.default_sender_option,
                    Box::new(autd3::r#async::AsyncSleeper) as _,
                ),
            };

            let num_devices = autd.num_devices();
            let mut sender = autd.sender(option, autd3::controller::FixedSchedule(sleeper));
            let res = match datagram.datagram.ok_or(AUTDProtoBufError::DataParseError)? {
                datagram::Datagram::Tuple(datagram_tuple) => {
                    sender.send(into_datagram_tuple(datagram_tuple)?).await
                }
                datagram::Datagram::Group(group) => {
                    let keys = group.keys;
                    if keys.len() != num_devices {
                        return Ok(Response::new(SendResponseLightweight {
                            err: true,
                            msg: "Length of keys must be the same as the number of devices"
                                .to_string(),
                        }));
                    }
                    let datagrams = group
                        .datagrams
                        .into_iter()
                        .map(into_datagram_tuple)
                        .collect::<Result<Vec<_>, _>>()?;
                    sender
                        .send(autd3_driver::datagram::Group::new(
                            |dev| {
                                let key = keys[dev.idx()];
                                if key < 0 { None } else { Some(key as usize) }
                            },
                            std::collections::HashMap::from_iter(datagrams.into_iter().enumerate()),
                        ))
                        .await
                }
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
