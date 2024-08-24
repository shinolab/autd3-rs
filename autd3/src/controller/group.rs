use std::{fmt::Debug, time::Duration};

use autd3_driver::{
    datagram::Datagram,
    error::AUTDInternalError,
    firmware::operation::{Operation, OperationGenerator},
    geometry::Device,
};

use super::{Controller, Link};
use crate::prelude::AUTDError;

use tracing;

pub struct GroupGuard<'a, K: PartialEq + Debug, L: Link> {
    pub(crate) cnt: &'a mut Controller<L>,
    pub(crate) keys: Vec<Option<K>>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) parallel_threshold: Option<usize>,
    pub(crate) operations: Vec<(Box<dyn Operation>, Box<dyn Operation>)>,
}

impl<'a, K: PartialEq + Debug, L: Link> GroupGuard<'a, K, L> {
    #[must_use]
    pub(crate) fn new(cnt: &'a mut Controller<L>, f: impl Fn(&Device) -> Option<K>) -> Self {
        Self {
            operations: (0..cnt.geometry.num_devices())
                .map(|_| {
                    (
                        Box::<dyn Operation>::default(),
                        Box::<dyn Operation>::default(),
                    )
                })
                .collect(),
            keys: cnt.geometry.devices().map(f).collect(),
            cnt,
            timeout: None,
            parallel_threshold: None,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    #[must_use]
    pub fn set<D: Datagram>(self, k: K, d: D) -> Result<Self, AUTDInternalError>
    where
        <<D as Datagram>::G as OperationGenerator>::O1: 'static,
        <<D as Datagram>::G as OperationGenerator>::O2: 'static,
    {
        let Self {
            keys,
            mut operations,
            cnt,
            timeout,
            parallel_threshold,
        } = self;

        if !keys
            .iter()
            .any(|key| key.as_ref().map(|kk| kk == &k).unwrap_or(false))
        {
            return Err(AUTDInternalError::UnkownKey(format!("{:?}", k)));
        }

        let timeout = match (timeout, d.timeout()) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (a, b) => a.or(b),
        };
        let parallel_threshold = match (parallel_threshold, d.parallel_threshold()) {
            (Some(t1), Some(t2)) => Some(t1.min(t2)),
            (a, b) => a.or(b),
        };

        let generator = d.operation_generator(&cnt.geometry)?;

        operations
            .iter_mut()
            .zip(keys.iter())
            .zip(cnt.geometry.devices())
            .for_each(|((op, key), dev)| {
                if let Some(kk) = key {
                    if kk == &k {
                        tracing::debug!("Generate operation for device {}", dev.idx());
                        let (op1, op2) = generator.generate(dev);
                        *op = (Box::new(op1) as Box<_>, Box::new(op2) as Box<_>);
                    }
                }
            });

        Ok(Self {
            cnt,
            keys,
            timeout,
            parallel_threshold,
            operations,
        })
    }

    pub async fn send(self) -> Result<(), AUTDError> {
        let Self {
            mut operations,
            cnt,
            timeout,
            parallel_threshold,
            ..
        } = self;
        cnt.send_impl(&mut operations, timeout, parallel_threshold)
            .await
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        datagram::{GainSTM, SwapSegment},
        defined::Hz,
        derive::{Modulation, Segment, TransitionMode},
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
            vec![Drive::null(); autd.geometry[0].num_transducers()],
            autd.link[0].fpga().drives(Segment::S0, 0)
        );

        assert_eq!(
            vec![Drive::null(); autd.geometry[1].num_transducers()],
            autd.link[1].fpga().drives(Segment::S0, 0)
        );
        assert_eq!(
            vec![0x80, 0x80],
            autd.link[1].fpga().modulation(Segment::S0)
        );

        assert_eq!(
            vec![
                Drive::new(Phase::new(0x00), EmitIntensity::new(0xFF));
                autd.geometry[2].num_transducers()
            ],
            autd.link[2].fpga().drives(Segment::S0, 0)
        );

        assert_eq!(
            *Sine::new(150. * Hz).calc()?,
            autd.link[3].fpga().modulation(Segment::S0)
        );
        assert_eq!(
            vec![
                Drive::new(Phase::new(0x00), EmitIntensity::new(0x80));
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives(Segment::S0, 0)
        );
        assert_eq!(
            vec![
                Drive::new(Phase::new(0x00), EmitIntensity::new(0x81));
                autd.geometry[3].num_transducers()
            ],
            autd.link[3].fpga().drives(Segment::S0, 1)
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

        let check = std::sync::Arc::new(std::sync::Mutex::new([false; 2]));
        autd.group(|dev| {
            check.lock().unwrap()[dev.idx()] = true;
            Some(0)
        })
        .set(0, (Static::new(), Null::new()))?
        .send()
        .await?;

        assert!(!check.lock().unwrap()[0]);
        assert!(check.lock().unwrap()[1]);

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
}
