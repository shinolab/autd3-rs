use autd3_driver::{derive::EmitIntensity, geometry::Vector3};

use crate::gain::Focus;

pub struct Line {
    pub start: Vector3,
    pub end: Vector3,
    pub num_points: usize,
    pub intensity: EmitIntensity,
}

pub struct LineIntoIter {
    start: Vector3,
    dir: Vector3,
    num_points: usize,
    intensity: EmitIntensity,
    i: usize,
}

impl Iterator for LineIntoIter {
    type Item = Focus;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.num_points {
            return None;
        }
        let f = Focus::new(self.start + self.dir * (self.i as f32 / (self.num_points - 1) as f32))
            .with_intensity(self.intensity);
        self.i += 1;
        Some(f)
    }
}

impl IntoIterator for Line {
    type Item = Focus;
    type IntoIter = LineIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        LineIntoIter {
            start: self.start,
            dir: self.end - self.start,
            num_points: self.num_points,
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

    #[test]
    fn line() {
        let length = 30.0 * mm;
        let line = Line {
            start: Vector3::new(0., -length / 2., 0.),
            end: Vector3::new(0., length / 2., 0.),
            num_points: 3,
            intensity: EmitIntensity::MAX,
        };

        let points = line.into_iter().collect::<Vec<_>>();

        assert_near_vector3!(&Vector3::new(0., -length / 2., 0.), points[0].pos());
        assert_near_vector3!(&Vector3::zeros(), points[1].pos());
        assert_near_vector3!(&Vector3::new(0., length / 2., 0.), points[2].pos());
    }
}
