/*
 * File: traits.rs
 * Project: src
 * Created Date: 30/06/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::geometry::IntoDevice;

use crate::pb::*;

pub trait ToMessage {
    type Message: prost::Message;

    fn to_msg(&self) -> Self::Message;
}

pub trait FromMessage<T: prost::Message> {
    fn from_msg(msg: &T) -> Self;
}

impl ToMessage for autd3_driver::geometry::Vector3 {
    type Message = Vector3;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        }
    }
}

impl ToMessage for autd3_driver::geometry::Quaternion {
    type Message = Quaternion;

    #[allow(clippy::unnecessary_cast)]
    fn to_msg(&self) -> Self::Message {
        Self::Message {
            w: self.w as _,
            x: self.coords.x as _,
            y: self.coords.y as _,
            z: self.coords.z as _,
        }
    }
}

impl ToMessage for autd3_driver::geometry::Geometry {
    type Message = Geometry;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            devices: self
                .iter()
                .map(|dev| geometry::Autd3 {
                    pos: Some(dev[0].position().to_msg()),
                    rot: Some(dev[0].rotation().to_msg()),
                    sound_speed: dev.sound_speed as _,
                    attenuation: dev.attenuation as _,
                })
                .collect(),
        }
    }
}

impl ToMessage for &[autd3_driver::geometry::Device] {
    type Message = Geometry;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            devices: self
                .iter()
                .map(|dev| geometry::Autd3 {
                    pos: Some(dev[0].position().to_msg()),
                    rot: Some(dev[0].rotation().to_msg()),
                    sound_speed: dev.sound_speed as _,
                    attenuation: dev.attenuation as _,
                })
                .collect(),
        }
    }
}

impl ToMessage for autd3_driver::cpu::TxDatagram {
    type Message = TxRawData;

    fn to_msg(&self) -> Self::Message {
        Self::Message {
            data: self.all_data().to_vec(),
            num_devices: self.num_devices() as _,
        }
    }
}

impl ToMessage for Vec<autd3_driver::cpu::RxMessage> {
    type Message = RxMessage;

    fn to_msg(&self) -> Self::Message {
        let mut data = vec![0; std::mem::size_of::<autd3_driver::cpu::RxMessage>() * self.len()];
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.as_ptr() as *const u8,
                data.as_mut_ptr(),
                data.len(),
            );
        }
        Self::Message { data }
    }
}

impl FromMessage<RxMessage> for Vec<autd3_driver::cpu::RxMessage> {
    fn from_msg(msg: &RxMessage) -> Self {
        let mut rx = vec![
            autd3_driver::cpu::RxMessage { ack: 0, data: 0 };
            msg.data.len() / std::mem::size_of::<autd3_driver::cpu::RxMessage>()
        ];
        unsafe {
            std::ptr::copy_nonoverlapping(msg.data.as_ptr(), rx.as_mut_ptr() as _, msg.data.len());
        }
        rx
    }
}

impl FromMessage<Vector3> for autd3_driver::geometry::Vector3 {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Vector3) -> Self {
        autd3_driver::geometry::Vector3::new(msg.x as _, msg.y as _, msg.z as _)
    }
}

impl FromMessage<Quaternion> for autd3_driver::geometry::UnitQuaternion {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Quaternion) -> Self {
        autd3_driver::geometry::UnitQuaternion::from_quaternion(
            autd3_driver::geometry::Quaternion::new(msg.w as _, msg.x as _, msg.y as _, msg.z as _),
        )
    }
}

impl FromMessage<Geometry> for autd3_driver::geometry::Geometry {
    fn from_msg(msg: &Geometry) -> Self {
        let devices = msg
            .devices
            .iter()
            .enumerate()
            .map(|(i, dev)| {
                let mut dev = autd3_driver::autd3_device::AUTD3::new(
                    autd3_driver::geometry::Vector3::from_msg(dev.pos.as_ref().unwrap()),
                )
                .with_rotation(autd3_driver::geometry::UnitQuaternion::from_msg(
                    dev.rot.as_ref().unwrap(),
                ))
                .into_device(i);
                dev.sound_speed = dev.sound_speed as _;
                dev.attenuation = dev.attenuation as _;
                dev
            })
            .collect();
        Self::new(devices)
    }
}

impl FromMessage<TxRawData> for autd3_driver::cpu::TxDatagram {
    fn from_msg(msg: &TxRawData) -> Self {
        let mut tx = autd3_driver::cpu::TxDatagram::new(msg.num_devices as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(
                msg.data.as_ptr(),
                tx.all_data_mut().as_mut_ptr(),
                msg.data.len(),
            );
        }
        tx
    }
}
