use std::{collections::HashMap, hash::Hash, time::Duration};

use autd3_driver::{
    datagram::Datagram,
    error::AUTDInternalError,
    geometry::Device,
    operation::{Operation, OperationHandler},
};

use super::Controller;
use super::Link;

type OpMap<K> = HashMap<K, (Box<dyn Operation>, Box<dyn Operation>)>;

#[allow(clippy::type_complexity)]
pub struct GroupGuard<'a, K: Hash + Eq + Clone, L: Link, F: Fn(&Device) -> Option<K>> {
    pub(crate) cnt: &'a mut Controller<L>,
    pub(crate) f: F,
    pub(crate) timeout: Option<Duration>,
    pub(crate) op: OpMap<K>,
}

impl<'a, K: Hash + Eq + Clone, L: Link, F: Fn(&Device) -> Option<K>> GroupGuard<'a, K, L, F> {
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
        let (op1, op2) = d.operation()?;
        self.set_boxed_op(k, Box::new(op1), Box::new(op2), timeout)
    }

    #[doc(hidden)]
    pub fn set_boxed_op(
        mut self,
        k: K,
        op1: Box<dyn autd3_driver::operation::Operation>,
        op2: Box<dyn autd3_driver::operation::Operation>,
        timeout: Option<Duration>,
    ) -> Result<Self, AUTDInternalError> {
        self.timeout = match (self.timeout, timeout) {
            (None, None) => None,
            (None, Some(t)) | (Some(t), None) => Some(t),
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
        };
        self.op.insert(k, (op1, op2));
        Ok(self)
    }

    fn push_enable_flags(&self) -> Vec<bool> {
        self.cnt
            .geometry
            .iter()
            .map(|dev| dev.enable)
            .collect::<Vec<_>>()
    }

    fn pop_enable_flags(&mut self, enable_flags_store: Vec<bool>) {
        self.cnt
            .geometry
            .iter_mut()
            .zip(enable_flags_store.iter())
            .for_each(|(dev, &enable)| dev.enable = enable);
    }

    fn get_enable_flags_map(&self) -> HashMap<K, Vec<bool>> {
        self.op
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
            .collect()
    }

    pub async fn send(mut self) -> Result<bool, AUTDInternalError> {
        let enable_flags_store = self.push_enable_flags();
        let enable_flags_map = self.get_enable_flags_map();

        self.op.iter_mut().try_for_each(|(k, (op1, op2))| {
            self.cnt.geometry.iter_mut().for_each(|dev| {
                dev.enable = enable_flags_map[k][dev.idx()];
            });
            OperationHandler::init(op1, op2, &self.cnt.geometry)
        })?;
        let r = loop {
            let start = std::time::Instant::now();
            self.op.iter_mut().try_for_each(|(k, (op1, op2))| {
                self.cnt.geometry.iter_mut().for_each(|dev| {
                    dev.enable = enable_flags_map[k][dev.idx()];
                });
                OperationHandler::pack(op1, op2, &self.cnt.geometry, &mut self.cnt.tx_buf)
            })?;

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
                self.cnt.geometry.iter_mut().for_each(|dev| {
                    dev.enable = enable_flags_map[k][dev.idx()];
                });
                OperationHandler::is_finished(op1, op2, &self.cnt.geometry)
            }) {
                break true;
            }
            if start.elapsed() < Duration::from_millis(1) {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        };

        self.pop_enable_flags(enable_flags_store);

        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::{autd3_device::AUTD3, geometry::Vector3};

    use crate::{
        controller::Controller,
        gain::{Null, Uniform},
        link::audit::Audit,
        modulation::{Sine, Static},
    };

    #[tokio::test]
    async fn test_group() {
        let mut autd = Controller::builder()
            .add_device(AUTD3::new(Vector3::zeros()))
            .add_device(AUTD3::new(Vector3::zeros()))
            .open_with(Audit::builder())
            .await
            .unwrap();

        autd.group(|dev| Some(dev.idx()))
            .set(0, (Static::new(), Null::new()))
            .unwrap()
            .set(1, (Sine::new(150.0), Uniform::new(0x80)))
            .unwrap()
            .send()
            .await
            .unwrap();
        {
            let m = autd.link[0].fpga().modulation();
            assert_eq!(2, m.len());
            assert!(m.iter().all(|&d| d == 0xFF));
            let v = autd.link[0].fpga().intensities_and_phases(0);
            assert!(v.iter().all(|d| d.0 == 0 && d.1 == 0));
        }
        {
            let m = autd.link[1].fpga().modulation();
            assert_eq!(80, m.len());
            let v = autd.link[1].fpga().intensities_and_phases(0);
            assert!(v.iter().all(|d| d.0 == 0x80 && d.1 == 0));
        }
    }

    #[tokio::test]
    async fn test_group_only_for_enabled() {
        let mut autd = Controller::builder()
            .add_device(AUTD3::new(Vector3::zeros()))
            .add_device(AUTD3::new(Vector3::zeros()))
            .open_with(Audit::builder())
            .await
            .unwrap();

        autd.geometry[0].enable = false;

        let check = std::sync::Arc::new(std::sync::Mutex::new(vec![false; 2]));
        autd.group(|dev| {
            check.lock().unwrap()[dev.idx()] = true;
            return Some(0);
        })
        .set(0, (Static::new(), Null::new()))
        .unwrap()
        .send()
        .await
        .unwrap();

        assert!(!check.lock().unwrap()[0]);
        assert!(check.lock().unwrap()[1]);
    }
}
