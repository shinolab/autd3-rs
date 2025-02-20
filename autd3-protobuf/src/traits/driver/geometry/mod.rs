use autd3_core::geometry::IntoDevice;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
    AUTDProtoBufError,
};

impl ToMessage for autd3_core::geometry::UnitVector3 {
    type Message = UnitVector3;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        })
    }
}

impl ToMessage for autd3_core::geometry::Point3 {
    type Message = Point3;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            x: self.x as _,
            y: self.y as _,
            z: self.z as _,
        })
    }
}

impl ToMessage for autd3_core::geometry::Quaternion {
    type Message = Quaternion;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            w: self.w as _,
            x: self.coords.x as _,
            y: self.coords.y as _,
            z: self.coords.z as _,
        })
    }
}

impl ToMessage for autd3_core::geometry::Geometry {
    type Message = Geometry;

    fn to_msg(
        &self,
        _: Option<&autd3_core::geometry::Geometry>,
    ) -> Result<Self::Message, AUTDProtoBufError> {
        Ok(Self::Message {
            devices: self
                .iter()
                .map(|dev| {
                    Ok(geometry::Autd3 {
                        pos: Some(dev[0].position().to_msg(None)?),
                        rot: Some(dev.rotation().to_msg(None)?),
                        sound_speed: dev.sound_speed as _,
                    })
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
        })
    }
}

impl FromMessage<UnitVector3> for autd3_core::geometry::UnitVector3 {
    fn from_msg(msg: UnitVector3) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_core::geometry::UnitVector3::new_normalize(
            autd3_core::geometry::Vector3::new(msg.x as _, msg.y as _, msg.z as _),
        ))
    }
}

impl FromMessage<Point3> for autd3_core::geometry::Point3 {
    fn from_msg(msg: Point3) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_core::geometry::Point3::new(
            msg.x as _, msg.y as _, msg.z as _,
        ))
    }
}

impl FromMessage<Quaternion> for autd3_core::geometry::UnitQuaternion {
    fn from_msg(msg: Quaternion) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_core::geometry::UnitQuaternion::from_quaternion(
            autd3_core::geometry::Quaternion::new(msg.w as _, msg.x as _, msg.y as _, msg.z as _),
        ))
    }
}

impl FromMessage<Geometry> for autd3_core::geometry::Geometry {
    fn from_msg(msg: Geometry) -> Result<Self, AUTDProtoBufError> {
        Ok(autd3_core::geometry::Geometry::new(
            msg.devices
                .into_iter()
                .enumerate()
                .map(|(i, dev_msg)| {
                    let pos = autd3_core::geometry::Point3::from_msg(
                        dev_msg.pos.ok_or(AUTDProtoBufError::DataParseError)?,
                    )?;
                    let rot = autd3_core::geometry::UnitQuaternion::from_msg(
                        dev_msg.rot.ok_or(AUTDProtoBufError::DataParseError)?,
                    )?;
                    let mut dev =
                        autd3_driver::autd3_device::AUTD3 { pos, rot }.into_device(i as _);
                    dev.sound_speed = dev_msg.sound_speed as _;
                    Ok(dev)
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
        ))
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
    fn point3() {
        let mut rng = rand::rng();
        let v = Point3::new(rng.random(), rng.random(), rng.random());
        let msg = v.to_msg(None).unwrap();
        let v2 = Point3::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);
    }

    #[test]
    fn unitvector3() {
        let mut rng = rand::rng();
        let v = UnitVector3::new_normalize(Vector3::new(rng.random(), rng.random(), rng.random()));
        let msg = v.to_msg(None).unwrap();
        let v2 = UnitVector3::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);
    }

    #[test]
    fn quaternion() {
        let mut rng = rand::rng();
        let q = UnitQuaternion::from_quaternion(Quaternion::new(
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
        ));
        let msg = q.to_msg(None).unwrap();
        let q2 = UnitQuaternion::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(q.w, q2.w);
        approx::assert_abs_diff_eq!(q.i, q2.i);
        approx::assert_abs_diff_eq!(q.j, q2.j);
        approx::assert_abs_diff_eq!(q.k, q2.k);
    }

    #[test]
    fn geometry() {
        let mut rng = rand::rng();
        let mut dev = AUTD3 {
            pos: Point3::new(rng.random(), rng.random(), rng.random()),
            rot: UnitQuaternion::identity(),
        }
        .into_device(0);
        dev.sound_speed = rng.random();
        let geometry = Geometry::new(vec![dev]);
        let msg = geometry.to_msg(None).unwrap();
        let geometry2 = Geometry::from_msg(msg).unwrap();
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
