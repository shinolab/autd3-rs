/*
 * File: mod.rs
 * Project: geometry
 * Created Date: 19/01/2024
 * Author: Shun Suzuki
 * -----
 * Last Modified: 20/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2024 Shun Suzuki. All rights reserved.
 *
 */

use autd3_driver::geometry::IntoDevice;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

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

impl FromMessage<Vector3> for autd3_driver::geometry::Vector3 {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Vector3) -> Option<Self> {
        Some(autd3_driver::geometry::Vector3::new(
            msg.x as _, msg.y as _, msg.z as _,
        ))
    }
}

impl FromMessage<Quaternion> for autd3_driver::geometry::UnitQuaternion {
    #[allow(clippy::unnecessary_cast)]
    fn from_msg(msg: &Quaternion) -> Option<Self> {
        Some(autd3_driver::geometry::UnitQuaternion::from_quaternion(
            autd3_driver::geometry::Quaternion::new(msg.w as _, msg.x as _, msg.y as _, msg.z as _),
        ))
    }
}

impl FromMessage<Geometry> for autd3_driver::geometry::Geometry {
    fn from_msg(msg: &Geometry) -> Option<Self> {
        msg.devices
            .iter()
            .enumerate()
            .map(|(i, dev_msg)| {
                let pos = dev_msg
                    .pos
                    .as_ref()
                    .map(autd3_driver::geometry::Vector3::from_msg)??;
                let rot = dev_msg
                    .rot
                    .as_ref()
                    .map(autd3_driver::geometry::UnitQuaternion::from_msg)??;
                let mut dev = autd3_driver::autd3_device::AUTD3::new(pos)
                    .with_rotation(rot)
                    .into_device(i);
                dev.sound_speed = dev_msg.sound_speed as _;
                dev.attenuation = dev_msg.attenuation as _;
                Some(dev)
            })
            .collect::<Option<Vec<_>>>()
            .map(Self::new)
    }
}