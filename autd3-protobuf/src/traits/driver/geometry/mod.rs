use crate::{AUTDProtoBufError, pb::*, traits::FromMessage};

impl From<autd3_core::geometry::UnitVector3> for UnitVector3 {
    fn from(value: autd3_core::geometry::UnitVector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<autd3_core::geometry::Point3> for Point3 {
    fn from(value: autd3_core::geometry::Point3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<autd3_core::geometry::UnitQuaternion> for Quaternion {
    fn from(value: autd3_core::geometry::UnitQuaternion) -> Self {
        Self {
            w: value.w,
            x: value.i,
            y: value.j,
            z: value.k,
        }
    }
}

impl From<&autd3_core::geometry::Geometry> for Geometry {
    fn from(value: &autd3_core::geometry::Geometry) -> Self {
        Self {
            devices: value
                .iter()
                .map(|dev| geometry::Autd3 {
                    pos: Some((*dev[0].position()).into()),
                    rot: Some((*dev.rotation()).into()),
                })
                .collect(),
        }
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
                .map(|dev_msg| {
                    let pos = dev_msg
                        .pos
                        .map(autd3_core::geometry::Point3::from_msg)
                        .transpose()?
                        .unwrap_or(autd3_core::geometry::Point3::origin());
                    let rot = dev_msg
                        .rot
                        .map(autd3_core::geometry::UnitQuaternion::from_msg)
                        .transpose()?
                        .unwrap_or(autd3_core::geometry::UnitQuaternion::identity());
                    Ok(autd3_driver::autd3_device::AUTD3 { pos, rot }.into())
                })
                .collect::<Result<Vec<_>, AUTDProtoBufError>>()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autd3_core::derive::Device;
    use autd3_driver::{
        autd3_device::AUTD3,
        geometry::{Geometry, Point3, Quaternion, UnitQuaternion, UnitVector3, Vector3},
    };
    use rand::Rng;

    #[test]
    fn point3() {
        let mut rng = rand::rng();
        let v = Point3::new(rng.random(), rng.random(), rng.random());
        let msg = v.into();
        let v2 = Point3::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(v.x, v2.x);
        approx::assert_abs_diff_eq!(v.y, v2.y);
        approx::assert_abs_diff_eq!(v.z, v2.z);
    }

    #[test]
    fn unitvector3() {
        let mut rng = rand::rng();
        let v = UnitVector3::new_normalize(Vector3::new(rng.random(), rng.random(), rng.random()));
        let msg = v.into();
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
        let msg = q.into();
        let q2 = UnitQuaternion::from_msg(msg).unwrap();
        approx::assert_abs_diff_eq!(q.w, q2.w);
        approx::assert_abs_diff_eq!(q.i, q2.i);
        approx::assert_abs_diff_eq!(q.j, q2.j);
        approx::assert_abs_diff_eq!(q.k, q2.k);
    }

    #[test]
    fn geometry() {
        let mut rng = rand::rng();
        let dev: Device = AUTD3 {
            pos: Point3::new(rng.random(), rng.random(), rng.random()),
            rot: UnitQuaternion::identity(),
        }
        .into();
        let geometry = Geometry::new(vec![dev]);
        let msg = (&geometry).into();
        let geometry2 = Geometry::from_msg(msg).unwrap();
        geometry
            .into_iter()
            .zip(geometry2.iter())
            .for_each(|(dev, dev2)| {
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
