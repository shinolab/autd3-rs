use std::f32::consts::PI;

use autd3_driver::{
    derive::EmitIntensity,
    geometry::{UnitVector3, Vector3},
};

use crate::gain::Focus;

pub struct Circle {
    pub center: Vector3,
    pub radius: f32,
    pub num_points: usize,
    pub n: UnitVector3,
    pub intensity: EmitIntensity,
}

pub struct CircleIntoIter {
    center: Vector3,
    radius: f32,
    num_points: usize,
    u: Vector3,
    v: Vector3,
    intensity: EmitIntensity,
    i: usize,
}

impl Iterator for CircleIntoIter {
    type Item = Focus;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.num_points {
            return None;
        }

        let theta = 2.0 * PI * self.i as f32 / self.num_points as f32;
        let f =
            Focus::new(self.center + self.radius * (theta.cos() * self.u + theta.sin() * self.v))
                .with_intensity(self.intensity);
        self.i += 1;
        Some(f)
    }
}

impl IntoIterator for Circle {
    type Item = Focus;
    type IntoIter = CircleIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let v = if self.n.dot(&Vector3::z()).abs() < 0.9 {
            Vector3::z()
        } else {
            Vector3::y()
        };
        let u = self.n.cross(&v).normalize();
        let v = self.n.cross(&u).normalize();
        CircleIntoIter {
            center: self.center,
            radius: self.radius,
            num_points: self.num_points,
            u,
            v,
            intensity: self.intensity,
            i: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use autd3_driver::defined::mm;

    use crate::assert_near_vector3;

    use super::*;

    #[rstest::rstest]
    #[case(
        vec![
            Vector3::new(0., -30.0 * mm, 0.),
            Vector3::new(0., 0., -30.0 * mm),
            Vector3::new(0., 30.0 * mm, 0.),
            Vector3::new(0., 0., 30.0 * mm),
        ]
        ,
        Vector3::x_axis()
    )]
    #[case(
        vec![
            Vector3::new(30.0 * mm, 0., 0.),
            Vector3::new(0., 0., -30.0 * mm),
            Vector3::new(-30.0 * mm, 0., 0.),
            Vector3::new(0., 0., 30.0 * mm),
        ]
        ,
        Vector3::y_axis()
    )]
    #[case(
        vec![
            Vector3::new(-30.0 * mm, 0., 0.),
            Vector3::new(0., -30.0 * mm, 0.),
            Vector3::new(30.0 * mm, 0., 0.),
            Vector3::new(0., 30.0 * mm, 0.),
        ]
        ,
        Vector3::z_axis()
    )]
    #[test]
    fn circle(#[case] expect: Vec<Vector3>, #[case] n: UnitVector3) {
        let circle = Circle {
            center: Vector3::zeros(),
            radius: 30.0 * mm,
            num_points: 4,
            n,
            intensity: EmitIntensity::MAX,
        };

        let points = circle.into_iter().collect::<Vec<_>>();
        assert_eq!(expect.len(), points.len());
        (expect.into_iter().zip(points.into_iter())).for_each(|(a, b)| {
            assert_near_vector3!(&a, b.pos());
        });
    }
}
