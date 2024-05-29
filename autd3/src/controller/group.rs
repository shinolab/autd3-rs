use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    time::Duration,
};

use autd3_driver::{
    datagram::Datagram,
    error::AUTDInternalError,
    firmware::operation::{Operation, OperationHandler},
    geometry::{Device, Geometry},
};

use super::Controller;
use super::Link;

type OpMap<K> = HashMap<K, Vec<(Box<dyn Operation>, Box<dyn Operation>)>>;

#[allow(clippy::type_complexity)]
pub struct GroupGuard<'a, K: Hash + Eq + Clone + Debug, L: Link, F: Fn(&Device) -> Option<K>> {
    pub(crate) cnt: &'a mut Controller<L>,
    pub(crate) f: F,
    pub(crate) timeout: Option<Duration>,
    pub(crate) op: OpMap<K>,
    pub(crate) enable_flags: Vec<bool>,
}

impl<'a, K: Hash + Eq + Clone + Debug, L: Link, F: Fn(&Device) -> Option<K>>
    GroupGuard<'a, K, L, F>
{
    pub(crate) fn new(cnt: &'a mut Controller<L>, f: F) -> Self {
        let enable_flags = cnt
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>();
        Self {
            cnt,
            f,
            timeout: None,
            op: OpMap::new(),
            enable_flags,
        }
    }

    pub fn set<D: Datagram<'a>>(self, k: K, d: D) -> Result<Self, AUTDInternalError>
    where
        D::O1: 'static,
        D::O2: 'static,
    {
        let timeout = d.timeout();
        let operations = {
            let gen = d.operation_generator(&self.cnt.geometry)?;
            OperationHandler::generate(gen, &self.cnt.geometry)
                .into_iter()
                .map(|(op1, op2)| (Box::new(op1) as Box<_>, Box::new(op2) as Box<_>))
                .collect()
        };
        Ok(self.set_boxed_op(k, operations, timeout))
    }

    #[doc(hidden)]
    pub fn set_boxed_op(
        mut self,
        k: K,
        op: Vec<(Box<dyn Operation>, Box<dyn Operation>)>,
        timeout: Option<Duration>,
    ) -> Self {
        self.timeout = match (self.timeout, timeout) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (a, b) => a.or(b),
        };
        self.op.insert(k, op);
        self
    }

    pub async fn send(mut self) -> Result<(), AUTDInternalError> {
        let specified_keys = self.op.keys().cloned().collect::<HashSet<_>>();
        let provided_keys = self
            .cnt
            .geometry
            .devices()
            .filter_map(|dev| (self.f)(dev))
            .collect::<HashSet<_>>();

        let unknown_keys = specified_keys
            .difference(&provided_keys)
            .collect::<Vec<_>>();
        if !unknown_keys.is_empty() {
            return Err(AUTDInternalError::UnkownKey(format!("{:?}", unknown_keys)));
        }
        let unspecified_keys = provided_keys
            .difference(&specified_keys)
            .collect::<Vec<_>>();
        if !unspecified_keys.is_empty() {
            return Err(AUTDInternalError::UnspecifiedKey(format!(
                "{:?}",
                unspecified_keys
            )));
        }

        let enable_flags_map = self
            .op
            .keys()
            .map(|k| {
                (
                    k.clone(),
                    self.cnt
                        .geometry
                        .iter()
                        .map(|dev| dev.enable && (self.f)(dev).map(|kk| &kk == k).unwrap_or(false))
                        .collect(),
                )
            })
            .collect();

        let set_enable_flag =
            |geometry: &mut Geometry, k: &K, enable_flags: &HashMap<K, Vec<bool>>| {
                geometry.iter_mut().for_each(|dev| {
                    dev.enable = enable_flags[k][dev.idx()];
                });
            };

        loop {
            self.op.iter_mut().try_for_each(|(k, op)| {
                set_enable_flag(&mut self.cnt.geometry, k, &enable_flags_map);
                if OperationHandler::is_done(op, &self.cnt.geometry) {
                    return Ok(());
                }
                OperationHandler::pack(
                    op,
                    &self.cnt.geometry,
                    &mut self.cnt.tx_buf,
                    self.cnt.parallel_threshold,
                )
            })?;

            let start = tokio::time::Instant::now();
            autd3_driver::link::send_receive(
                &mut self.cnt.link,
                &self.cnt.tx_buf,
                &mut self.cnt.rx_buf,
                self.timeout,
            )
            .await?;

            if self.op.iter_mut().all(|(k, op)| {
                set_enable_flag(&mut self.cnt.geometry, k, &enable_flags_map);
                OperationHandler::is_done(op, &self.cnt.geometry)
            }) {
                break;
            }
            tokio::time::sleep_until(start + Duration::from_millis(1)).await;
        }

        Ok(())
    }
}

impl<'a, K: Hash + Eq + Clone + Debug, L: Link, F: Fn(&Device) -> Option<K>> Drop
    for GroupGuard<'a, K, L, F>
{
    fn drop(&mut self) {
        self.cnt
            .geometry
            .iter_mut()
            .zip(self.enable_flags.iter())
            .for_each(|(dev, &enable)| dev.enable = enable);
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        datagram::{GainSTM, SwapSegment},
        defined::Hz,
        derive::{Drive, EmitIntensity, Modulation, Phase, Segment, TransitionMode},
        error::AUTDInternalError,
    };

    use crate::{
        controller::tests::create_controller,
        gain::{Null, Uniform},
        modulation::{Sine, Static},
    };

    #[tokio::test]
    async fn test_group() -> anyhow::Result<()> {
        let mut autd = create_controller(4).await?;

        autd.send(Uniform::new(0xFF)).await?;

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
                GainSTM::from_freq(
                    1. * Hz,
                    [Uniform::new(0x80), Uniform::new(0x81)].into_iter(),
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
            Sine::new(150. * Hz).calc(&autd.geometry)?,
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

        autd.link.down();
        assert_eq!(
            Err(AUTDInternalError::SendDataFailed),
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
            Err(autd3_driver::error::AUTDInternalError::InvalidSegmentTransition),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .set(
                    1,
                    SwapSegment::FocusSTM(Segment::S1, TransitionMode::SyncIdx),
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
            Err(AUTDInternalError::UnkownKey("[2]".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .set(2, Null::new())?
                .send()
                .await
        );

        Ok(())
    }

    #[tokio::test]
    async fn unspecified_key() -> anyhow::Result<()> {
        let mut autd = create_controller(2).await?;

        assert_eq!(
            Err(AUTDInternalError::UnspecifiedKey("[1]".to_owned())),
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .send()
                .await
        );

        Ok(())
    }
}
