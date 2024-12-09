use std::{fmt::Debug, hash::Hash, time::Duration};

use autd3_driver::{
    datagram::Datagram,
    error::AUTDInternalError,
    firmware::operation::{Operation, OperationGenerator},
    geometry::Device,
};
use itertools::Itertools;

use super::{Controller, Link};
use crate::prelude::AUTDError;

use tracing;

/// A struct for grouping devices and sending different data to each group. See also [`Controller::group`].
#[allow(clippy::type_complexity)]
pub struct Group<'a, K: PartialEq + Debug, L: Link> {
    pub(crate) cnt: &'a mut Controller<L>,
    pub(crate) keys: Vec<Option<K>>,
    pub(crate) done: Vec<bool>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) parallel_threshold: Option<usize>,
    pub(crate) operations: Vec<Option<(Box<dyn Operation>, Box<dyn Operation>)>>,
}

impl<'a, K: PartialEq + Debug, L: Link> Group<'a, K, L> {
    #[must_use]
    pub(crate) fn new(cnt: &'a mut Controller<L>, f: impl Fn(&Device) -> Option<K>) -> Self {
        let keys = cnt.geometry.devices().map(f).collect::<Vec<_>>();
        let done = keys.iter().map(Option::is_none).collect();
        Self {
            operations: cnt.geometry.devices().map(|_| None).collect(),
            keys,
            done,
            cnt,
            timeout: None,
            parallel_threshold: None,
        }
    }

    /// Set the `data` to be sent to the devices corresponding to the `key`.
    ///
    /// # Errors
    ///
    /// - Returns [`AUTDInternalError::UnkownKey`] if the `key` is not specified in the [`Controller::group`].
    /// - Returns [`AUTDInternalError::KeyIsAlreadyUsed`] if the `key` is already used previous [`Group::set`].
    ///
    /// [`AUTDInternalError::UnkownKey`]: autd3_driver::error::AUTDInternalError::UnkownKey
    /// [`AUTDInternalError::KeyIsAlreadyUsed`]: autd3_driver::error::AUTDInternalError::KeyIsAlreadyUsed
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn set<D: Datagram>(self, key: K, data: D) -> Result<Self, AUTDInternalError>
    where
        <<D as Datagram>::G as OperationGenerator>::O1: 'static,
        <<D as Datagram>::G as OperationGenerator>::O2: 'static,
    {
        let Self {
            keys,
            mut done,
            mut operations,
            cnt,
            timeout,
            parallel_threshold,
        } = self;

        if !keys
            .iter()
            .any(|k| k.as_ref().map(|kk| kk == &key).unwrap_or(false))
        {
            return Err(AUTDInternalError::UnkownKey(format!("{:?}", key)));
        }

        let timeout = timeout.into_iter().chain(data.timeout()).max();
        let parallel_threshold = parallel_threshold
            .into_iter()
            .chain(data.parallel_threshold())
            .min();

        // set enable flag for each device
        // This is not required for the operation except `Gain`s which cannot be calculated independently for each device, such as `autd3-gain-holo`.
        let enable_store = cnt
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>();
        cnt.geometry
            .devices_mut()
            .zip(keys.iter())
            .for_each(|(dev, k)| {
                dev.enable = k.as_ref().is_some_and(|kk| kk == &key);
            });
        let mut generator = data.operation_generator(&cnt.geometry)?;
        cnt.geometry
            .iter_mut()
            .zip(enable_store)
            .for_each(|(dev, enable)| {
                dev.enable = enable;
            });

        operations
            .iter_mut()
            .zip(keys.iter())
            .zip(cnt.geometry.devices())
            .zip(done.iter_mut())
            .filter(|(((_, k), _), _)| k.as_ref().is_some_and(|kk| kk == &key))
            .try_for_each(|(((op, _), dev), done)| {
                if *done {
                    return Err(AUTDInternalError::KeyIsAlreadyUsed(format!("{:?}", key)));
                }
                *done = true;
                tracing::debug!("Generate operation for device {}", dev.idx());
                let (op1, op2) = generator.generate(dev);
                *op = Some((Box::new(op1) as Box<_>, Box::new(op2) as Box<_>));
                Ok(())
            })?;

        Ok(Self {
            cnt,
            keys,
            done,
            timeout,
            parallel_threshold,
            operations,
        })
    }

    /// Send the data to the devices.
    ///
    /// # Errors
    ///
    /// Returns [`AUTDInternalError::UnusedKey`] if the data is not specified for the key by [`Group::set`].
    ///
    /// [`AUTDInternalError::UnusedKey`]: autd3_driver::error::AUTDInternalError::UnusedKey
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send(self) -> Result<(), AUTDError> {
        let Self {
            operations,
            cnt,
            keys,
            done,
            timeout,
            parallel_threshold,
            ..
        } = self;

        if !done.iter().all(|&d| d) {
            return Err(AUTDError::Internal(AUTDInternalError::UnusedKey(
                keys.into_iter()
                    .zip(done.into_iter())
                    .filter(|(_, d)| !*d)
                    .map(|(k, _)| format!("{:?}", k.unwrap()))
                    .join(", "),
            )));
        }

        cnt.link.trace(timeout, parallel_threshold);
        cnt.timer
            .send(
                &cnt.geometry,
                &mut cnt.tx_buf,
                &mut cnt.rx_buf,
                &mut cnt.link,
                operations
                    .into_iter()
                    .map(|op| op.unwrap_or_default())
                    .collect::<Vec<_>>(),
                timeout,
                parallel_threshold,
            )
            .await
    }
}

impl<L: Link> Controller<L> {
    /// Group the devices by given function and send different data to each group.
    ///
    /// If the key is `None`, nothing is done for the devices corresponding to the key.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # tokio_test::block_on(async {
    /// let mut autd = Controller::builder((0..3).map(|_| AUTD3::new(Vector3::zeros()))).open(Nop::builder()).await?;
    ///
    /// autd.group(|dev| match dev.idx() {
    ///    0 => Some("static"),
    ///    2 => Some("sine"),
    ///   _ => None,
    /// })
    /// .set("static", Static::new())?
    /// .set("sine", Sine::new(150 * Hz))?
    /// .send().await?;
    /// # Result::<(), AUTDError>::Ok(())
    /// # });
    /// ```
    #[must_use]
    pub fn group<K: Hash + Eq + Clone + Debug, F: Fn(&Device) -> Option<K>>(
        &mut self,
        f: F,
    ) -> Group<K, L> {
        Group::new(self, f)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use autd3_driver::{
        datagram::{GainSTM, SwapSegment},
        defined::Hz,
        derive::*,
        error::AUTDInternalError,
        firmware::fpga::{Drive, EmitIntensity, Phase},
    };

    use crate::{
        controller::tests::create_controller,
        gain::{Null, Uniform},
        modulation::{Sine, Static},
        prelude::AUTDError,
    };

    #[tokio::test]
    async fn test_group() -> anyhow::Result<()> {
        let mut autd = create_controller(4).await?;

        autd.send(Uniform::new(EmitIntensity::new(0xFF))).await?;

        autd.group(|dev| match dev.idx() {
            0 | 1 | 3 => Some(dev.idx()),
            _ => None,
        })
        .set(0, Null::new())?
        .set(1, (Static::with_intensity(0x80), Null::new()))?
        .set(
            3,
            (
                Sine::new(150. * Hz),
                GainSTM::new(
                    1. * Hz,
                    [
                        Uniform::new(EmitIntensity::new(0x80)),
                        Uniform::new(EmitIntensity::new(0x81)),
                    ]
                    .into_iter(),
                )?,
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
                Drive::new(Phase::ZERO, EmitIntensity::new(0xFF));
                autd.geometry[2].num_transducers()
            ],
            autd.link[2].fpga().drives_at(Segment::S0, 0)
        );

        assert_eq!(
            *Sine::new(150. * Hz).calc()?,
            autd.link[3].fpga().modulation_buffer(Segment::S0)
        );
        assert_eq!(
            vec![
                Drive::new(Phase::ZERO, EmitIntensity::new(0x80));
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives_at(Segment::S0, 0)
        );
        assert_eq!(
            vec![
                Drive::new(Phase::ZERO, EmitIntensity::new(0x81));
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
                .set(0, Null::new())?
                .send()
                .await
        );

        autd.link_mut().down();
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::SendDataFailed)),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .send()
                .await
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_err() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
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
        .set(1, Static::with_intensity(0x80))?
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

    #[derive(Gain, Debug)]
    pub struct TestGain {
        test: Arc<Mutex<Vec<bool>>>,
    }

    impl Gain for TestGain {
        type G = Null;

        fn init(
            self,
            geometry: &Geometry,
            _filter: Option<&HashMap<usize, BitVec<u32>>>,
        ) -> Result<Self::G, AUTDInternalError> {
            geometry.iter().for_each(|dev| {
                self.test.lock().unwrap()[dev.idx()] = dev.enable;
            });
            Ok(Null::new())
        }
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
            Some(AUTDInternalError::UnkownKey("2".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .set(2, Null::new())
                .err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn already_used_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Some(AUTDInternalError::KeyIsAlreadyUsed("1".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .set(1, Null::new())?
                .set(1, Null::new())
                .err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn unused_key() -> anyhow::Result<()> {
        let mut autd = create_controller(3).await?;

        assert_eq!(
            Some(AUTDError::Internal(AUTDInternalError::UnusedKey(
                "0, 2".to_owned()
            ))),
            autd.group(|dev| Some(dev.idx()))
                .set(1, Null::new())?
                .send()
                .await
                .err()
        );

        Ok(())
    }
}
