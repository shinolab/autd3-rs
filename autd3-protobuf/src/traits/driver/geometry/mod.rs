use autd3_core::geometry::IntoDevice;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_core::geometry::UnitVector3 {
    type Message = UnitVector3;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        }
    }
}

impl ToMessage for autd3_core::geometry::Point3 {
    type Message = Point3;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        }
    }
}

impl ToMessage for autd3_core::geometry::Quaternion {
    type Message = Quaternion;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            w: self.w as _,
            x: self.coords.x as _,
            y: self.coords.y as _,
            z: self.coords.z as _,
        }
    }
}

impl ToMessage for autd3_core::geometry::Geometry {
    type Message = Geometry;

    fn to_msg(&self, _: Option<&autd3_core::geometry::Geometry>) -> Self::Message {
        Self::Message {
            devices: self
                .iter()
                .map(|dev| geometry::Autd3 {
                    pos: Some(dev[0].position().to_msg(None)),
                    rot: Some(dev.rotation().to_msg(None)),
                    sound_speed: dev.sound_speed as _,
                })
                .collect(),
            default_parallel_threshold: self.default_parallel_threshold() as _,
        }
    }
}

impl FromMessage<Option<UnitVector3>> for autd3_core::geometry::UnitVector3 {
    fn from_msg(msg: &Option<UnitVector3>) -> Result<Self, AUTDProtoBufError> {
        msg.as_ref()
            .map(|msg| {
                autd3_core::geometry::UnitVector3::new_unchecked(
                    autd3_core::geometry::Vector3::new(msg.x as _, msg.y as _, msg.z as _),
                )
            })
            .ok_or(AUTDProtoBufError::DataParseError)
    }
}

impl FromMessage<Option<Point3>> for autd3_core::geometry::Point3 {
    fn from_msg(msg: &Option<Point3>) -> Result<Self, AUTDProtoBufError> {
        msg.as_ref()
            .map(|msg| autd3_core::geometry::Point3::new(msg.x as _, msg.y as _, msg.z as _))
            .ok_or(AUTDProtoBufError::DataParseError)
    }
}

impl FromMessage<Option<Quaternion>> for autd3_core::geometry::UnitQuaternion {
    fn from_msg(msg: &Option<Quaternion>) -> Result<Self, AUTDProtoBufError> {
        msg.as_ref()
            .map(|msg| {
                autd3_core::geometry::UnitQuaternion::from_quaternion(
                    autd3_core::geometry::Quaternion::new(
                        msg.w as _, msg.x as _, msg.y as _, msg.z as _,
                    ),
                )
            })
            .ok_or(AUTDProtoBufError::DataParseError)
    }
}

impl FromMessage<Geometry> for autd3_core::geometry::Geometry {
    fn from_msg(msg: &Geometry) -> Result<Self, AUTDProtoBufError> {
        msg.devices
            .iter()
            .enumerate()
            .map(|(i, dev_msg)| {
                let pos = autd3_core::geometry::Point3::from_msg(&dev_msg.pos)?;
                let rot = autd3_core::geometry::UnitQuaternion::from_msg(&dev_msg.rot)?;
                let mut dev = autd3_driver::autd3_device::AUTD3::new(pos)
                    .with_rotation(rot)
                    .into_device(i as _);
                dev.sound_speed = dev_msg.sound_speed as _;
                Ok(dev)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|devices| Self::new(devices, msg.default_parallel_threshold as _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{Geometry, Point3, Quaternion, UnitQuaternion, UnitVector3, Vector3},
    };
    use rand::Rng;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn point3() {
        let mut rng = rand::thread_rng();
        let v = Point3::new(rng.gen(), rng.gen(), rng.gen());
        let msg = v.to_msg(None);
        let v2 = Point3::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);

        assert!(Point3::from_msg(&None).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn unitvector3() {
        let mut rng = rand::thread_rng();
        let v = UnitVector3::new_normalize(Vector3::new(rng.gen(), rng.gen(), rng.gen()));
        let msg = v.to_msg(None);
        let v2 = UnitVector3::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);

        assert!(UnitVector3::from_msg(&None).is_err());
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
        let q2 = UnitQuaternion::from_msg(&Some(msg)).unwrap();
        approx::assert_abs_diff_eq!(q.w, q2.w);
        approx::assert_abs_diff_eq!(q.i, q2.i);
        approx::assert_abs_diff_eq!(q.j, q2.j);
        approx::assert_abs_diff_eq!(q.k, q2.k);

        assert!(UnitQuaternion::from_msg(&None).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn geometry() {
        let mut rng = rand::thread_rng();
        let mut dev = AUTD3::new(Point3::new(rng.gen(), rng.gen(), rng.gen())).into_device(0);
        dev.sound_speed = rng.gen();
        let geometry = Geometry::new(vec![dev], 4);
        let msg = geometry.to_msg(None);
        let geometry2 = Geometry::from_msg(&msg).unwrap();
        assert_eq!(
            geometry.default_parallel_threshold(),
            geometry2.default_parallel_threshold()
        );
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
