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

type OpMap<K> = HashMap<K, (Box<dyn Operation>, Box<dyn Operation>)>;

#[allow(clippy::type_complexity)]
pub struct GroupGuard<'a, K: Hash + Eq + Clone + Debug, L: Link, F: Fn(&Device) -> Option<K>> {
    pub(crate) cnt: &'a mut Controller<L>,
    pub(crate) f: F,
    pub(crate) timeout: Option<Duration>,
    pub(crate) op: OpMap<K>,
}

impl<'a, K: Hash + Eq + Clone + Debug, L: Link, F: Fn(&Device) -> Option<K>>
    GroupGuard<'a, K, L, F>
{
    pub(crate) fn new(cnt: &'a mut Controller<L>, f: F) -> Self {
        Self {
            cnt,
            f,
            timeout: None,
            op: OpMap::new(),
        }
    }

    pub fn set<D: Datagram>(self, k: K, d: D) -> Result<Self, AUTDInternalError>
    where
        D::O1: 'static,
        D::O2: 'static,
    {
        let timeout = d.timeout();
        let (op1, op2) = d.operation();
        self.set_boxed_op(k, Box::new(op1), Box::new(op2), timeout)
    }

    #[doc(hidden)]
    pub fn set_boxed_op(
        mut self,
        k: K,
        op1: Box<dyn autd3_driver::firmware::operation::Operation>,
        op2: Box<dyn autd3_driver::firmware::operation::Operation>,
        timeout: Option<Duration>,
    ) -> Result<Self, AUTDInternalError> {
        self.timeout = match (self.timeout, timeout) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (a, b) => a.or(b),
        };
        self.op.insert(k, (op1, op2));
        Ok(self)
    }

    pub async fn send(mut self) -> Result<bool, AUTDInternalError> {
        let enable_flags_store = self
            .cnt
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>();

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

        self.op.iter_mut().try_for_each(|(k, (op1, op2))| {
            set_enable_flag(&mut self.cnt.geometry, k, &enable_flags_map);
            OperationHandler::init(op1, op2, &self.cnt.geometry)
        })?;
        let r = loop {
            self.op.iter_mut().try_for_each(|(k, (op1, op2))| {
                set_enable_flag(&mut self.cnt.geometry, k, &enable_flags_map);
                if OperationHandler::is_finished(op1, op2, &self.cnt.geometry) {
                    return Ok(());
                }
                OperationHandler::pack(op1, op2, &self.cnt.geometry, &mut self.cnt.tx_buf)
            })?;

            let start = tokio::time::Instant::now();
            if !autd3_driver::link::send_receive(
                &mut self.cnt.link,
                &self.cnt.tx_buf,
                &mut self.cnt.rx_buf,
                self.timeout,
            )
            .await?
            {
                break false;
            }
            if self.op.iter_mut().all(|(k, (op1, op2))| {
                set_enable_flag(&mut self.cnt.geometry, k, &enable_flags_map);
                OperationHandler::is_finished(op1, op2, &self.cnt.geometry)
            }) {
                break true;
            }
            tokio::time::sleep_until(start + Duration::from_millis(1)).await;
        };

        self.cnt
            .geometry
            .iter_mut()
            .zip(enable_flags_store.iter())
            .for_each(|(dev, &enable)| dev.enable = enable);

        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{
        datagram::{ChangeFocusSTMSegment, GainSTM},
        derive::{Gain, GainFilter, Modulation, Segment, TransitionMode},
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
                Sine::new(150.),
                GainSTM::from_freq(1.)
                    .add_gain(Uniform::new(0x80))
                    .add_gain(Uniform::new(0x81)),
            ),
        )?
        .send()
        .await?;

        assert_eq!(
            Null::new().calc(&autd.geometry, GainFilter::All)?[&0],
            autd.link[0].fpga().drives(Segment::S0, 0)
        );

        assert_eq!(
            Null::new().calc(&autd.geometry, GainFilter::All)?[&0],
            autd.link[1].fpga().drives(Segment::S0, 0)
        );
        assert_eq!(
            Static::with_intensity(0x80).calc(&autd.geometry)?[&0],
            autd.link[1].fpga().modulation(Segment::S0)
        );

        assert_eq!(
            Uniform::new(0xFF).calc(&autd.geometry, GainFilter::All)?[&2],
            autd.link[2].fpga().drives(Segment::S0, 0)
        );

        assert_eq!(
            Sine::new(150.).calc(&autd.geometry)?[&0],
            autd.link[3].fpga().modulation(Segment::S0)
        );
        assert_eq!(
            Uniform::new(0x80).calc(&autd.geometry, GainFilter::All)?[&3],
            autd.link[3].fpga().drives(Segment::S0, 0)
        );
        assert_eq!(
            Uniform::new(0x81).calc(&autd.geometry, GainFilter::All)?[&3],
            autd.link[3].fpga().drives(Segment::S0, 1)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_send_failed() -> anyhow::Result<()> {
        let mut autd = create_controller(1).await?;
        assert!(
            autd.group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .send()
                .await?
        );

        autd.link.down();
        assert!(
            !autd
                .group(|dev| Some(dev.idx()))
                .set(0, Null::new())?
                .send()
                .await?
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
                    ChangeFocusSTMSegment::new(Segment::S1, TransitionMode::SyncIdx),
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
