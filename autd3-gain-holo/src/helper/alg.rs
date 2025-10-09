use crate::{MatrixXc, helper::propagate};

use autd3_core::{
    acoustics::directivity::Directivity,
    environment::Environment,
    gain::TransducerMask,
    geometry::{Complex, Geometry, Point3},
};

struct Ptr(*mut Complex);
impl Ptr {
    #[inline]
    fn write(&mut self, value: Complex) {
        unsafe {
            *self.0 = value;
            self.0 = self.0.add(1);
        }
    }

    #[inline]
    fn add(&self, i: usize) -> Self {
        Self(unsafe { self.0.add(i) })
    }
}
unsafe impl Send for Ptr {}
unsafe impl Sync for Ptr {}

#[cfg(feature = "use_nalgebra")]
fn uninit_mat(row: usize, col: usize) -> MatrixXc {
    use nalgebra::{Dyn, VecStorage};
    use std::mem::{ManuallyDrop, MaybeUninit};

    MatrixXc::from_data(unsafe {
        let mut data = Vec::<MaybeUninit<Complex>>::new();
        let length = row * col;
        data.reserve_exact(length);
        data.resize_with(length, MaybeUninit::uninit);
        let uninit = VecStorage::new(Dyn(row), Dyn(col), data);
        let vec: Vec<_> = uninit.into();
        let mut md = ManuallyDrop::new(vec);
        let new_data = Vec::from_raw_parts(md.as_mut_ptr() as *mut _, md.len(), md.capacity());
        VecStorage::new(Dyn(row), Dyn(col), new_data)
    })
}

#[cfg(not(feature = "use_nalgebra"))]
fn uninit_mat(row: usize, col: usize) -> MatrixXc {
    MatrixXc::zeros(row, col)
}

pub fn generate_propagation_matrix<D: Directivity>(
    geometry: &Geometry,
    env: &Environment,
    foci: &[Point3],
    filter: &TransducerMask,
) -> MatrixXc {
    let num_transducers = [0]
        .into_iter()
        .chain(geometry.iter().scan(0, |state, dev| {
            *state += filter.num_enabled_transducers(dev);
            Some(*state)
        }))
        .collect::<Vec<_>>();
    let n = num_transducers.last().copied().unwrap();

    if filter.is_all_enabled() {
        let mut r = uninit_mat(foci.len(), n);
        let ptr = Ptr(r.as_mut_ptr());
        geometry.iter().for_each(move |dev| {
            let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
            dev.iter().for_each(move |tr| {
                foci.iter().for_each(|f| {
                    ptr.write(propagate::<D>(
                        tr,
                        env.wavenumber(),
                        dev.axial_direction(),
                        *f,
                    ));
                });
            });
        });
        r
    } else {
        let mut r = uninit_mat(foci.len(), n);
        let ptr = Ptr(r.as_mut_ptr());
        geometry
            .iter()
            .filter(|dev| filter.has_enabled(dev))
            .for_each(move |dev| {
                let mut ptr = ptr.add(foci.len() * num_transducers[dev.idx()]);
                dev.iter().for_each(move |tr| {
                    if filter.is_enabled(tr) {
                        foci.iter().for_each(|f| {
                            ptr.write(propagate::<D>(
                                tr,
                                env.wavenumber(),
                                dev.axial_direction(),
                                *f,
                            ));
                        });
                    }
                });
            });
        r
    }
}

pub fn gen_back_prop(m: usize, n: usize, transfer: &MatrixXc) -> MatrixXc {
    MatrixXc::from_vec(
        m,
        n,
        (0..n)
            .flat_map(|i| {
                let x = 1.0
                    / transfer
                        .rows(i, 1)
                        .iter()
                        .map(|x| x.norm_sqr())
                        .sum::<f32>();
                (0..m).map(move |j| transfer[(i, j)].conj() * x)
            })
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use autd3_core::{
        acoustics::directivity::Sphere,
        derive::{Device, Transducer},
        environment::Environment,
        gain::{DeviceTransducerMask, TransducerMask},
        geometry::{Point3, UnitQuaternion},
    };

    use super::*;

    fn create_geometry(num_devices: usize) -> Geometry {
        Geometry::new(
            (0..num_devices)
                .map(|_| {
                    Device::new(
                        UnitQuaternion::identity(),
                        vec![Transducer::new(Point3::new(0., 0., 0.))],
                    )
                })
                .collect(),
        )
    }

    fn check_matrix(
        geometry: &Geometry,
        env: &Environment,
        foci: &[Point3],
        filter: &TransducerMask,
        m: &MatrixXc,
    ) {
        let expected_cols: usize = geometry
            .iter()
            .map(|dev| filter.num_enabled_transducers(dev))
            .sum();
        assert_eq!(m.nrows(), foci.len());
        assert_eq!(m.ncols(), expected_cols);

        let mut col = 0;
        geometry.iter().for_each(|dev| {
            let dir = dev.axial_direction();
            dev.iter().for_each(|tr| {
                if filter.is_enabled(tr) {
                    foci.iter().enumerate().for_each(|(i, f)| {
                        let exp = propagate::<Sphere>(tr, env.wavenumber(), dir, *f);
                        assert_eq!(m[(i, col)], exp);
                    });
                    col += 1;
                }
            });
        });
    }

    #[test]
    fn generate_propagation_matrix_all_enabled() {
        let geometry = create_geometry(2);
        let env = Environment::new();
        let foci = vec![
            Point3::new(0.0, 0.0, 100.0),
            Point3::new(10.0, 0.0, 150.0),
            Point3::new(0.0, -20.0, 200.0),
        ];
        let filter = TransducerMask::AllEnabled;
        let m = generate_propagation_matrix::<Sphere>(&geometry, &env, &foci, &filter);
        check_matrix(&geometry, &env, &foci, &filter, &m);
    }

    #[test]
    fn generate_propagation_matrix_masked() {
        let geometry = create_geometry(2);
        let env = Environment::new();
        let foci = vec![
            Point3::new(5.0, 0.0, 80.0),
            Point3::new(-10.0, 10.0, 160.0),
            Point3::new(0.0, 15.0, 240.0),
        ];
        let filter = TransducerMask::new(vec![
            DeviceTransducerMask::AllEnabled,
            DeviceTransducerMask::AllDisabled,
        ]);
        let m = generate_propagation_matrix::<Sphere>(&geometry, &env, &foci, &filter);
        check_matrix(&geometry, &env, &foci, &filter, &m);
    }
}
