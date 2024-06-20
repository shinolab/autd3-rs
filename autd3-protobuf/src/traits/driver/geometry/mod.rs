use autd3_driver::geometry::IntoDevice;

use crate::{
    pb::*,
    traits::{FromMessage, ToMessage},
};

impl ToMessage for autd3_driver::geometry::Vector3 {
    type Message = Vector3;

    #[allow(clippy::unnecessary_cast)]
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

    #[allow(clippy::unnecessary_cast)]
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
                Some(dev)
            })
            .collect::<Option<Vec<_>>>()
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
    fn test_vector3() {
        let mut rng = rand::thread_rng();
        let v = Vector3::new(rng.gen(), rng.gen(), rng.gen());
        let msg = v.to_msg(None);
        let v2 = Vector3::from_msg(&msg).unwrap();
        assert_approx_eq::assert_approx_eq!(v.x, v2.x);
        assert_approx_eq::assert_approx_eq!(v.y, v2.y);
        assert_approx_eq::assert_approx_eq!(v.z, v2.z);
    }

    #[test]
    fn test_quaternion() {
        let mut rng = rand::thread_rng();
        let q = UnitQuaternion::from_quaternion(Quaternion::new(
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
        ));
        let msg = q.to_msg(None);
        let q2 = UnitQuaternion::from_msg(&msg).unwrap();
        assert_approx_eq::assert_approx_eq!(q.w, q2.w);
        assert_approx_eq::assert_approx_eq!(q.i, q2.i);
        assert_approx_eq::assert_approx_eq!(q.j, q2.j);
        assert_approx_eq::assert_approx_eq!(q.k, q2.k);
    }

    #[test]
    fn test_geometry() {
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
                assert_approx_eq::assert_approx_eq!(dev.sound_speed, dev2.sound_speed);
                assert_approx_eq::assert_approx_eq!(dev.rotation().w, dev2.rotation().w);
                assert_approx_eq::assert_approx_eq!(dev.rotation().i, dev2.rotation().i);
                assert_approx_eq::assert_approx_eq!(dev.rotation().j, dev2.rotation().j);
                assert_approx_eq::assert_approx_eq!(dev.rotation().k, dev2.rotation().k);
                dev.iter().zip(dev2.iter()).for_each(|(t, t2)| {
                    assert_approx_eq::assert_approx_eq!(t.position().x, t2.position().x);
                    assert_approx_eq::assert_approx_eq!(t.position().y, t2.position().y);
                    assert_approx_eq::assert_approx_eq!(t.position().z, t2.position().z);
                });
            });
    }
}
