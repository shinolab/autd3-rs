/*
 * File: group.rs
 * Project: controller
 * Created Date: 05/10/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 01/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

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
    pub fn set<D: Datagram>(mut self, k: K, d: D) -> Result<Self, AUTDInternalError>
    where
        D::O1: 'static,
        D::O2: 'static,
    {
        self.timeout = match (self.timeout, d.timeout()) {
            (None, None) => None,
            (None, Some(t)) => Some(t),
            (Some(t), None) => Some(t),
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
        };
        let (op1, op2) = d.operation()?;
        self.op.insert(k, (Box::new(op1), Box::new(op2)));
        Ok(self)
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
            (None, Some(t)) => Some(t),
            (Some(t), None) => Some(t),
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
                        .map(|dev| {
                            if !dev.enable {
                                return false;
                            }
                            if let Some(kk) = (self.f)(dev) {
                                kk == *k
                            } else {
                                false
                            }
                        })
                        .collect(),
                )
            })
            .collect()
    }

    #[cfg(not(feature = "sync"))]
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

            if !self
                .cnt
                .link
                .send_receive(
                    &self.cnt.tx_buf,
                    &mut self.cnt.rx_buf,
                    self.timeout,
                    self.cnt.ignore_ack,
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
                std::thread::sleep(Duration::from_millis(1));
            }
        };

        self.pop_enable_flags(enable_flags_store);

        Ok(r)
    }

    #[cfg(feature = "sync")]
    pub fn send(mut self) -> Result<bool, AUTDInternalError> {
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

            if !self.cnt.link.send_receive(
                &self.cnt.tx_buf,
                &mut self.cnt.rx_buf,
                self.timeout,
                self.cnt.ignore_ack,
            )? {
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
                std::thread::sleep(Duration::from_millis(1));
            }
        };

        self.pop_enable_flags(enable_flags_store);

        Ok(r)
    }
}
