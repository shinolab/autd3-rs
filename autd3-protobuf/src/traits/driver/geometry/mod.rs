use autd3_driver::geometry::IntoDevice;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_driver::geometry::Vector3 {
    type Message = Vector3;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        }
    }
}

impl ToMessage for autd3_driver::geometry::Quaternion {
    type Message = Quaternion;

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
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

    fn to_msg(&self, _: Option<&autd3_driver::geometry::Geometry>) -> Self::Message {
        Self::Message {
            devices: self
                .iter()
                .map(|dev| geometry::Autd3 {
                    pos: Some(dev[0].position().to_msg(None)),
                    rot: Some(dev.rotation().to_msg(None)),
                    sound_speed: dev.sound_speed as _,
                })
                .collect(),
        }
    }
}

impl FromMessage<Option<Vector3>> for autd3_driver::geometry::Vector3 {
    fn from_msg(msg: &Option<Vector3>) -> Result<Self, AUTDProtoBufError> {
        match msg {
            Some(msg) => Ok(autd3_driver::geometry::Vector3::new(
                msg.x as _, msg.y as _, msg.z as _,
            )),
            None => Err(AUTDProtoBufError::DataParseError),
        }
    }
}

impl FromMessage<Quaternion> for autd3_driver::geometry::UnitQuaternion {
    fn from_msg(msg: &Quaternion) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_driver::geometry::UnitQuaternion::from_quaternion(
            autd3_driver::geometry::Quaternion::new(msg.w as _, msg.x as _, msg.y as _, msg.z as _),
        ))
    }
}

impl FromMessage<Geometry> for autd3_driver::geometry::Geometry {
    fn from_msg(msg: &Geometry) -> Result<Self, AUTDProtoBufError> {
        msg.devices
            .iter()
            .enumerate()
            .map(|(i, dev_msg)| {
                let pos = autd3_driver::geometry::Vector3::from_msg(&dev_msg.pos)?;
                let rot = dev_msg
                    .rot
                    .as_ref()
                    .ok_or(AUTDProtoBufError::DataParseError)
                    .map(autd3_driver::geometry::UnitQuaternion::from_msg)??;
                let mut dev = autd3_driver::autd3_device::AUTD3::new(pos)
                    .with_rotation(rot)
                    .into_device(i as _);
                dev.sound_speed = dev_msg.sound_speed as _;
                Ok(dev)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{Geometry, Quaternion, UnitQuaternion, Vector3},
    };
    use rand::Rng;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn vector3() {
        let mut rng = rand::thread_rng();
        let v = Vector3::new(rng.gen(), rng.gen(), rng.gen());
        let msg = v.to_msg(None);
        let v2 = Vector3::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn parse_error() {
        assert!(Vector3::from_msg(&None).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn quaternion() {
        let mut rng = rand::thread_rng();
        let q = UnitQuaternion::from_quaternion(Quaternion::new(
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        ));
        let msg = q.to_msg(None);
        let q2 = UnitQuaternion::from_msg(&msg).unwrap();
        approx::assert_abs_diff_eq!(q.w, q2.w);
        approx::assert_abs_diff_eq!(q.i, q2.i);
        approx::assert_abs_diff_eq!(q.j, q2.j);
        approx::assert_abs_diff_eq!(q.k, q2.k);
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn geometry() {
        let mut rng = rand::thread_rng();
        let mut dev = AUTD3::new(Vector3::new(rng.gen(), rng.gen(), rng.gen())).into_device(0);
        dev.sound_speed = rng.gen();
        let geometry = Geometry::new(vec![dev]);
        let msg = geometry.to_msg(None);
        let geometry2 = Geometry::from_msg(&msg).unwrap();
        geometry
            .iter()
            .zip(geometry2.iter())
            .for_each(|(dev, dev2)| {
                approx::assert_abs_diff_eq!(dev.sound_speed, dev2.sound_speed);
                approx::assert_abs_diff_eq!(dev.rotation().w, dev2.rotation().w);
                approx::assert_abs_diff_eq!(dev.rotation().i, dev2.rotation().i);
                approx::assert_abs_diff_eq!(dev.rotation().j, dev2.rotation().j);
                approx::assert_abs_diff_eq!(dev.rotation().k, dev2.rotation().k);
                dev.iter().zip(dev2.iter()).for_each(|(t, t2)| {
                    approx::assert_abs_diff_eq!(t.position().x, t2.position().x);
                    approx::assert_abs_diff_eq!(t.position().y, t2.position().y);
                    approx::assert_abs_diff_eq!(t.position().z, t2.position().z);
                });
            });
    }
}
