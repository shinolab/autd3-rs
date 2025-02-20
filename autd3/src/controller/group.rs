use std::{collections::HashMap, fmt::Debug, hash::Hash, time::Duration};

use autd3_core::{derive::DatagramOption, link::Link};
use autd3_driver::{
    datagram::Datagram,
    error::AUTDDriverError,
    firmware::operation::{Operation, OperationGenerator},
    geometry::Device,
};
use bit_vec::BitVec;
use itertools::Itertools;
use spin_sleep::SpinSleeper;

use crate::error::AUTDError;

use super::{
    Controller, Sleep,
    sender::{Sender, SenderOption},
};

impl<L: Link> Controller<L> {
    /// Groups the devices by given function and send different data to each group. This is a shortcut for [`Sender::group_send`].
    pub fn group_send<K, D, F>(
        &mut self,
        key_map: F,
        datagram_map: HashMap<K, D>,
    ) -> Result<(), AUTDError>
    where
        K: Hash + Eq + Debug,
        D: Datagram,
        F: Fn(&Device) -> Option<K>,
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        self.sender(SenderOption::<SpinSleeper>::default())
            .group_send(key_map, datagram_map)
    }
}

impl<L: Link, S: Sleep> Sender<'_, L, S> {
    /// Groups the devices by given function and send different data to each group.
    ///
    /// If the key is `None`, nothing is done for the devices corresponding to the key.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// # use autd3::prelude::*;
    /// use autd3::datagram::IntoBoxedDatagram;
    ///
    /// # fn main() -> Result<(), AUTDError> {
    /// let mut autd = Controller::open((0..3).map(|_| AUTD3::default()), Nop::new())?;
    ///
    /// autd.sender(SenderOption::<SpinSleeper>::default())
    /// .group_send(
    ///     |dev| match dev.idx() {
    ///         0 => Some("static"),
    ///         2 => Some("sine"),
    ///         _ => None,
    ///     },
    ///     HashMap::from([
    ///         ("static", Static::default().into_boxed()),
    ///         (
    ///             "sine",
    ///             Sine {
    ///                 freq: 150 * Hz,
    ///                 option: Default::default(),
    ///             }
    ///             .into_boxed(),
    ///         ),
    ///     ]),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn group_send<K, D, F>(
        &mut self,
        key_map: F,
        datagram_map: HashMap<K, D>,
    ) -> Result<(), AUTDError>
    where
        K: Hash + Eq + Debug,
        D: Datagram,
        F: Fn(&Device) -> Option<K>,
        AUTDDriverError: From<D::Error>,
        D::G: OperationGenerator,
        AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
            + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
    {
        let mut datagram_map = datagram_map;

        let filters = {
            let num_devices = self.geometry.iter().len();
            let mut filters: HashMap<K, BitVec> = HashMap::new();
            self.geometry.devices().for_each(|dev| {
                if let Some(key) = key_map(dev) {
                    if let Some(v) = filters.get_mut(&key) {
                        v.set(dev.idx(), true);
                    } else {
                        filters.insert(key, BitVec::from_fn(num_devices, |i| i == dev.idx()));
                    }
                }
            });
            filters
        };

        let enable_store = self
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>();

        let mut operations: Vec<_> = self.geometry.devices().map(|_| None).collect();
        let mut datagram_option = DatagramOption {
            timeout: Duration::ZERO,
            parallel_threshold: usize::MAX,
        };
        filters
            .into_iter()
            .try_for_each(|(k, filter)| -> Result<(), AUTDError> {
                {
                    // set enable flag for each device
                    // This is not required for the operation except `Gain`s which cannot be calculated independently for each device, such as `autd3-gain-holo`.
                    self.geometry.devices_mut().for_each(|dev| {
                        dev.enable = filter[dev.idx()];
                    });

                    let datagram = datagram_map
                        .remove(&k)
                        .ok_or(AUTDError::UnkownKey(format!("{:?}", k)))?;
                    datagram_option = DatagramOption {
                        timeout: datagram_option.timeout.max(datagram.option().timeout),
                        parallel_threshold: datagram_option
                            .parallel_threshold
                            .min(datagram.option().parallel_threshold),
                    };
                    let parallel = self.option.parallel.is_parallel(
                        self.geometry.num_devices(),
                        datagram.option().parallel_threshold,
                    );
                    let mut generator = datagram
                        .operation_generator(self.geometry, parallel)
                        .map_err(AUTDDriverError::from)?;

                    // restore enable flag
                    self.geometry
                        .iter_mut()
                        .zip(enable_store.iter())
                        .for_each(|(dev, &enable)| {
                            dev.enable = enable;
                        });

                    operations
                        .iter_mut()
                        .zip(self.geometry.devices())
                        .filter(|(_, dev)| filter[dev.idx()])
                        .for_each(|(op, dev)| {
                            tracing::debug!("Generate operation for device {}", dev.idx());
                            let (op1, op2) = generator.generate(dev);
                            *op = Some((op1, op2));
                        });
                    Ok(())
                }
            })?;

        if !datagram_map.is_empty() {
            return Err(AUTDError::UnusedKey(
                datagram_map.keys().map(|k| format!("{:?}", k)).join(", "),
            ));
        }

        let timeout = self.option.timeout.unwrap_or(datagram_option.timeout);
        let parallel = self.option.parallel.is_parallel(
            self.geometry.num_devices(),
            datagram_option.parallel_threshold,
        );
        tracing::debug!("timeout: {:?}, parallel: {:?}", timeout, parallel);
        Ok(self.send_impl(operations, timeout, parallel)?)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use autd3_core::{derive::*, link::LinkError};
    use autd3_driver::{
        datagram::{GainSTM, IntoBoxedDatagram, SwapSegment},
        defined::Hz,
        error::AUTDDriverError,
        firmware::fpga::{Drive, EmitIntensity, Phase},
    };
    use spin_sleep::SpinSleeper;

    use crate::{
        controller::{
            ParallelMode, SenderOption,
            tests::{TestGain, create_controller},
        },
        error::AUTDError,
        gain::{Null, Uniform},
        modulation::{Sine, Static},
    };

    #[rstest::rstest]
    #[case(ParallelMode::On)]
    #[case(ParallelMode::Off)]
    #[test]
    fn test_group(#[case] parallel: ParallelMode) -> anyhow::Result<()> {
        let mut autd = create_controller(4)?;

        autd.send(Uniform {
            intensity: EmitIntensity(0xFF),
            phase: Phase::ZERO,
        })?;

        autd.sender(SenderOption::<SpinSleeper> {
            parallel,
            ..Default::default()
        })
        .group_send(
            |dev| match dev.idx() {
                0 | 1 | 3 => Some(dev.idx()),
                _ => None,
            },
            HashMap::from([
                (0, Null {}.into_boxed()),
                (1, (Static { intensity: 0x80 }, Null {}).into_boxed()),
                (
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
                    )
                        .into_boxed(),
                ),
            ]),
        )?;

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

    #[test]
    fn test_send_failed() -> anyhow::Result<()> {
        let mut autd = create_controller(1)?;
        assert_eq!(
            Ok(()),
            autd.group_send(|_| Some(()), HashMap::from([((), Null {})]))
        );

        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDError::Driver(LinkError::new("broken").into())),
            autd.group_send(|_| Some(()), HashMap::from([((), Null {})]))
        );

        Ok(())
    }

    #[test]
    fn test_send_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2)?;

        assert_eq!(
            Err(AUTDError::Driver(AUTDDriverError::InvalidSegmentTransition)),
            autd.group_send(
                |dev| Some(dev.idx()),
                HashMap::from([
                    (0, Null {}.into_boxed(),),
                    (
                        1,
                        SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx).into_boxed()
                    )
                ])
            )
        );

        Ok(())
    }

    #[test]
    fn test_group_only_for_enabled() -> anyhow::Result<()> {
        let mut autd = create_controller(2)?;

        autd.geometry[0].enable = false;

        let check = Arc::new(Mutex::new([false; 2]));
        autd.group_send(
            |dev| {
                check.lock().unwrap()[dev.idx()] = true;
                Some(())
            },
            HashMap::from([((), Static { intensity: 0x80 })]),
        )?;

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

    #[test]
    fn test_group_only_for_enabled_gain() -> anyhow::Result<()> {
        let mut autd = create_controller(3)?;

        let test = Arc::new(Mutex::new(vec![false; 3]));
        autd.group_send(
            |dev| match dev.idx() {
                0 | 2 => Some(()),
                _ => None,
            },
            HashMap::from([((), TestGain { test: test.clone() })]),
        )?;

        assert!(test.lock().unwrap()[0]);
        assert!(!test.lock().unwrap()[1]);
        assert!(test.lock().unwrap()[2]);

        Ok(())
    }

    #[test]
    fn unknown_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2)?;

        assert_eq!(
            Some(AUTDError::UnkownKey("1".to_owned())),
            autd.group_send(|dev| Some(dev.idx()), HashMap::from([(0, Null {})]))
                .err()
        );

        Ok(())
    }

    #[test]
    fn unused_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2)?;

        assert_eq!(
            Some(AUTDError::UnusedKey("2".to_owned())),
            autd.group_send(
                |dev| Some(dev.idx()),
                HashMap::from([(0, Null {}), (1, Null {}), (2, Null {})])
            )
            .err()
        );

        Ok(())
    }
}
