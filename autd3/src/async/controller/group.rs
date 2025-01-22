use std::{borrow::BorrowMut, fmt::Debug, hash::Hash, time::Duration};

use autd3_core::{derive::DatagramOption, link::AsyncLink};
use autd3_driver::{
    datagram::Datagram,
    error::AUTDDriverError,
    firmware::operation::{BoxedOperation, Operation, OperationGenerator},
    geometry::Device,
};
use itertools::Itertools;

use crate::controller::SenderOption;

use super::{
    sender::{AsyncSleeper, Sender},
    AsyncSleep, Controller,
};

/// A struct for grouping devices and sending different data to each group. See also [`Sender::group`].
pub struct Group<
    'a,
    S: AsyncSleep + 'a,
    L: AsyncLink + 'a,
    T: BorrowMut<Sender<'a, L, S>>,
    K: PartialEq + Debug,
> {
    pub(crate) sender: T,
    pub(crate) keys: Vec<Option<K>>,
    pub(crate) done: Vec<bool>,
    pub(crate) datagram_option: DatagramOption,
    pub(crate) operations: Vec<Option<(BoxedOperation, BoxedOperation)>>,
    _phantom: std::marker::PhantomData<&'a (S, L)>,
}

impl<'a, S: AsyncSleep, L: AsyncLink, T: BorrowMut<Sender<'a, L, S>>, K: PartialEq + Debug>
    Group<'a, S, L, T, K>
{
    #[must_use]
    pub(crate) fn new(sender: T, f: impl Fn(&Device) -> Option<K>) -> Self {
        let keys = sender
            .borrow()
            .geometry
            .devices()
            .map(f)
            .collect::<Vec<_>>();
        let done = keys.iter().map(Option::is_none).collect();
        Self {
            operations: sender.borrow().geometry.devices().map(|_| None).collect(),
            keys,
            done,
            sender,
            datagram_option: DatagramOption {
                timeout: Duration::ZERO,
                parallel_threshold: usize::MAX,
            },
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the `data` to be sent to the devices corresponding to the `key`.
    ///
    /// # Errors
    ///
    /// - Returns [`AUTDDriverError::UnkownKey`] if the `key` is not specified in the [`Controller::group`].
    /// - Returns [`AUTDDriverError::KeyIsAlreadyUsed`] if the `key` is already used previous [`Group::set`].
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn set<D: Datagram>(self, key: K, data: D) -> Result<Self, AUTDDriverError>
    where
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        <<D as Datagram>::G as OperationGenerator>::O1: 'static,
        <<D as Datagram>::G as OperationGenerator>::O2: 'static,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let Self {
            keys,
            mut done,
            mut operations,
            mut sender,
            datagram_option,
            ..
        } = self;

        if !keys
            .iter()
            .any(|k| k.as_ref().map(|kk| kk == &key).unwrap_or(false))
        {
            return Err(AUTDDriverError::UnkownKey(format!("{:?}", key)));
        }

        let datagram_option = DatagramOption {
            timeout: sender
                .borrow()
                .option
                .timeout
                .unwrap_or(datagram_option.timeout.max(data.option().timeout)),
            parallel_threshold: sender.borrow().option.parallel_threshold.unwrap_or(
                datagram_option
                    .parallel_threshold
                    .min(data.option().parallel_threshold),
            ),
        };

        // set enable flag for each device
        // This is not required for the operation except `Gain`s which cannot be calculated independently for each device, such as `autd3-gain-holo`.
        let enable_store = sender
            .borrow()
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>();
        sender
            .borrow_mut()
            .geometry
            .devices_mut()
            .zip(keys.iter())
            .for_each(|(dev, k)| {
                dev.enable = k.as_ref().is_some_and(|kk| kk == &key);
            });
        let mut generator = data.operation_generator(sender.borrow().geometry, &datagram_option)?;
        sender
            .borrow_mut()
            .geometry
            .iter_mut()
            .zip(enable_store)
            .for_each(|(dev, enable)| {
                dev.enable = enable;
            });

        operations
            .iter_mut()
            .zip(keys.iter())
            .zip(sender.borrow().geometry.devices())
            .zip(done.iter_mut())
            .filter(|(((_, k), _), _)| k.as_ref().is_some_and(|kk| kk == &key))
            .try_for_each(|(((op, _), dev), done)| {
                if *done {
                    return Err(AUTDDriverError::KeyIsAlreadyUsed(format!("{:?}", key)));
                }
                *done = true;
                tracing::debug!("Generate operation for device {}", dev.idx());
                let (op1, op2) = generator.generate(dev);
                *op = Some((BoxedOperation::new(op1), BoxedOperation::new(op2)));
                Ok(())
            })?;

        Ok(Self {
            sender,
            keys,
            done,
            datagram_option,
            operations,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Send the data to the devices.
    ///
    /// # Errors
    ///
    /// Returns [`AUTDDriverError::UnusedKey`] if the data is not specified for the key by [`Group::set`].
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send(self) -> Result<(), AUTDDriverError> {
        let Self {
            operations,
            mut sender,
            keys,
            done,
            datagram_option,
            ..
        } = self;

        if !done.iter().all(|&d| d) {
            return Err(AUTDDriverError::UnusedKey(
                keys.into_iter()
                    .zip(done.into_iter())
                    .filter(|(_, d)| !*d)
                    .map(|(k, _)| format!("{:?}", k.unwrap()))
                    .join(", "),
            ));
        }

        sender
            .borrow_mut()
            .send_impl(
                operations
                    .into_iter()
                    .map(|op| op.unwrap_or_default())
                    .collect::<Vec<_>>(),
                &datagram_option,
            )
            .await
    }
}

impl<'a, L: AsyncLink, S: AsyncSleep> Sender<'a, L, S> {
    /// Group the devices by given function and send different data to each group.
    ///
    /// If the key is `None`, nothing is done for the devices corresponding to the key.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # fn main() -> Result<(), AUTDError> {
    /// let mut autd = Controller::open((0..3).map(|_| AUTD3::default()), Nop::builder())?;
    ///
    /// autd.group(|dev| match dev.idx() {
    ///    0 => Some("static"),
    ///    2 => Some("sine"),
    ///   _ => None,
    /// })
    /// .set("static", Static::default())?
    /// .set("sine", Sine { freq: 150 * Hz, option: Default::default() })?
    /// .send()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn group<K: Hash + Eq + Clone + Debug, F: Fn(&Device) -> Option<K>>(
        &'a mut self,
        f: F,
    ) -> Group<'a, S, L, &'a mut Self, K> {
        Group::new(self, f)
    }
}

impl<L: AsyncLink> Controller<L> {
    /// Group the devices by given function and send different data to each group.
    ///
    /// If the key is `None`, nothing is done for the devices corresponding to the key.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # fn main() -> Result<(), AUTDError> {
    /// let mut autd = Controller::open((0..3).map(|_| AUTD3::default()), Nop::builder())?;
    ///
    /// autd.group(|dev| match dev.idx() {
    ///    0 => Some("static"),
    ///    2 => Some("sine"),
    ///   _ => None,
    /// })
    /// .set("static", Static::default())?
    /// .set("sine", Sine { freq: 150 * Hz, option: Default::default() })?
    /// .send()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn group<K: Hash + Eq + Clone + Debug, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> Group<'_, AsyncSleeper, L, Sender<'_, L, AsyncSleeper>, K> {
        Group::new(
            self.sender(AsyncSleeper::default(), SenderOption::default()),
            f,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use autd3_core::derive::*;
    use autd3_driver::{
        datagram::{GainSTM, SwapSegment},
        defined::Hz,
        error::AUTDDriverError,
        firmware::fpga::{Drive, EmitIntensity, Phase},
    };

    use crate::{
        controller::tests::TestGain,
        gain::{Null, Uniform},
        modulation::{Sine, Static},
        r#async::{controller::tests::create_controller, AsyncSleeper},
    };

    #[tokio::test]
    async fn test_group() -> anyhow::Result<()> {
        let mut autd = create_controller(4).await?;

        autd.send(Uniform {
            intensity: EmitIntensity(0xFF),
            phase: Phase::ZERO,
        })
        .await?;

        autd.group(|dev| match dev.idx() {
            0 | 1 | 3 => Some(dev.idx()),
            _ => None,
        })
        .set(0, Null {})?
        .set(1, (Static { intensity: 0x80 }, Null {}))?
        .set(
            3,
            (
                Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                },
                GainSTM {
                    gains: vec![
                        Uniform {
                            intensity: EmitIntensity(0x80),
                            phase: Phase::ZERO,
                        },
                        Uniform {
                            intensity: EmitIntensity(0x81),
                            phase: Phase::ZERO,
                        },
                    ],
                    config: 1. * Hz,
                    option: Default::default(),
                },
            ),
        )?
        .send()
        .await?;

        assert_eq!(
            vec![Drive::NULL; autd.geometry[0].num_transducers()],
            autd.link[0].fpga().drives_at(Segment::S0, 0)
        );

        assert_eq!(
            vec![Drive::NULL; autd.geometry[1].num_transducers()],
            autd.link[1].fpga().drives_at(Segment::S0, 0)
        );
        assert_eq!(
            vec![0x80, 0x80],
            autd.link[1].fpga().modulation_buffer(Segment::S0)
        );

        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0xFF)
                };
                autd.geometry[2].num_transducers()
            ],
            autd.link[2].fpga().drives_at(Segment::S0, 0)
        );

        assert_eq!(
            *Sine {
                freq: 150. * Hz,
                option: Default::default(),
            }
            .calc()?,
            autd.link[3].fpga().modulation_buffer(Segment::S0)
        );
        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0x80)
                };
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives_at(Segment::S0, 0)
        );
        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0x81)
                };
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives_at(Segment::S0, 1)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_group_sender() -> anyhow::Result<()> {
        let mut autd = create_controller(4).await?;

        let mut sender = autd.sender(AsyncSleeper::default(), Default::default());

        sender
            .send(Uniform {
                intensity: EmitIntensity(0xFF),
                phase: Phase::ZERO,
            })
            .await?;

        sender
            .group(|dev| match dev.idx() {
                0 | 1 | 3 => Some(dev.idx()),
                _ => None,
            })
            .set(0, Null {})?
            .set(1, (Static { intensity: 0x80 }, Null {}))?
            .set(
                3,
                (
                    Sine {
                        freq: 150. * Hz,
                        option: Default::default(),
                    },
                    GainSTM {
                        gains: vec![
                            Uniform {
                                intensity: EmitIntensity(0x80),
                                phase: Phase::ZERO,
                            },
                            Uniform {
                                intensity: EmitIntensity(0x81),
                                phase: Phase::ZERO,
                            },
                        ],
                        config: 1. * Hz,
                        option: Default::default(),
                    },
                ),
            )?
            .send()
            .await?;

        assert_eq!(
            vec![Drive::NULL; autd.geometry[0].num_transducers()],
            autd.link[0].fpga().drives_at(Segment::S0, 0)
        );

        assert_eq!(
            vec![Drive::NULL; autd.geometry[1].num_transducers()],
            autd.link[1].fpga().drives_at(Segment::S0, 0)
        );
        assert_eq!(
            vec![0x80, 0x80],
            autd.link[1].fpga().modulation_buffer(Segment::S0)
        );

        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0xFF)
                };
                autd.geometry[2].num_transducers()
            ],
            autd.link[2].fpga().drives_at(Segment::S0, 0)
        );

        assert_eq!(
            *Sine {
                freq: 150. * Hz,
                option: Default::default(),
            }
            .calc()?,
            autd.link[3].fpga().modulation_buffer(Segment::S0)
        );
        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0x80)
                };
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives_at(Segment::S0, 0)
        );
        assert_eq!(
            vec![
                Drive {
                    phase: Phase::ZERO,
                    intensity: EmitIntensity(0x81)
                };
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives_at(Segment::S0, 1)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_failed() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;
        assert_eq!(
            Ok(()),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null {})?
                .send()
                .await
        );

        autd.link_mut().down();
        assert_eq!(
            Err(AUTDDriverError::SendDataFailed),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null {})?
                .send()
                .await
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Err(AUTDDriverError::InvalidSegmentTransition),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null {})?
                .set(
                    1,
                    SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx),
                )?
                .send()
                .await
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_group_only_for_enabled() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        autd.geometry[0].enable = false;

        let check = Arc::new(Mutex::new([false; 2]));
        autd.group(|dev| {
            check.lock().unwrap()[dev.idx()] = true;
            Some(dev.idx())
        })
        .set(1, Static { intensity: 0x80 })?
        .send()
        .await?;

        assert!(!check.lock().unwrap()[0]);
        assert!(check.lock().unwrap()[1]);

        assert_eq!(
            vec![0xFF, 0xFF],
            autd.link[0].fpga().modulation_buffer(Segment::S0)
        );
        assert_eq!(
            vec![0x80, 0x80],
            autd.link[1].fpga().modulation_buffer(Segment::S0)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_group_only_for_enabled_gain() -> anyhow::Result<()> {
        let mut autd = create_controller(3).await?;

        let test = Arc::new(Mutex::new(vec![false; 3]));
        autd.group(|dev| match dev.idx() {
            0 | 2 => Some(0),
            _ => None,
        })
        .set(0, TestGain { test: test.clone() })?
        .send()
        .await?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

        Ok(())
    }

    #[tokio::test]
    async fn unknown_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Some(AUTDDriverError::UnkownKey("2".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null {})?
                .set(2, Null {})
                .err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn already_used_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Some(AUTDDriverError::KeyIsAlreadyUsed("1".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null {})?
                .set(1, Null {})?
                .set(1, Null {})
                .err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn unused_key() -> anyhow::Result<()> {
        let mut autd = create_controller(3).await?;

        assert_eq!(
            Some(AUTDDriverError::UnusedKey("0, 2".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(1, Null {})?
                .send()
                .await
                .err()
        );

        Ok(())
    }
}
