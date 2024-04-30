use std::collections::HashMap;

use autd3_driver::{
    datagram::GainFilter, derive::Phase, error::AUTDInternalError, firmware::fpga::Drive,
    geometry::Geometry,
};
use nalgebra::ComplexField;

use crate::EmissionConstraint;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_holo {
    ($directivity:tt, $backend:tt, $t:ty) => {
        impl<$directivity, $backend> $t
        where
            $directivity: autd3_driver::acoustics::directivity::Directivity,
            $backend: $crate::LinAlgBackend<$directivity>,
        {
            /// Add focus
            pub fn add_focus(self, focus: Vector3, amp: $crate::amp::Amplitude) -> Self {
                let mut foci = self.foci;
                let mut amps = self.amps;
                foci.push(focus);
                amps.push(amp);
                Self { foci, amps, ..self }
            }

            /// Set constraint
            pub fn with_constraint(self, constraint: EmissionConstraint) -> Self {
                Self { constraint, ..self }
            }

            /// Add foci
            pub fn add_foci_from_iter(
                self,
                iter: impl IntoIterator<Item = (Vector3, $crate::amp::Amplitude)>,
            ) -> Self {
                let mut foci = self.foci;
                let mut amps = self.amps;
                iter.into_iter().for_each(|(focus, amp)| {
                    foci.push(focus);
                    amps.push(amp);
                });
                Self { foci, amps, ..self }
            }

            pub fn foci(
                &self,
            ) -> std::iter::Zip<
                std::slice::Iter<'_, Vector3>,
                std::slice::Iter<'_, $crate::amp::Amplitude>,
            > {
                self.foci.iter().zip(self.amps.iter())
            }

            pub const fn constraint(&self) -> EmissionConstraint {
                self.constraint
            }

            fn amps_as_slice(&self) -> &[f64] {
                unsafe {
                    std::slice::from_raw_parts(self.amps.as_ptr() as *const f64, self.amps.len())
                }
            }
        }
    };

    ($directivity:tt, $t:ty) => {
        impl<$directivity> $t
        where
            $directivity: autd3_driver::acoustics::directivity::Directivity,
        {
            /// Add focus
            pub fn add_focus(self, focus: Vector3, amp: $crate::amp::Amplitude) -> Self {
                let mut foci = self.foci;
                let mut amps = self.amps;
                foci.push(focus);
                amps.push(amp);
                Self { foci, amps, ..self }
            }

            /// Set constraint
            pub fn with_constraint(self, constraint: EmissionConstraint) -> Self {
                Self { constraint, ..self }
            }

            /// Add foci
            pub fn add_foci_from_iter(
                self,
                iter: impl IntoIterator<Item = (Vector3, $crate::amp::Amplitude)>,
            ) -> Self {
                let mut foci = self.foci;
                let mut amps = self.amps;
                iter.into_iter().for_each(|(focus, amp)| {
                    foci.push(focus);
                    amps.push(amp);
                });
                Self { foci, amps, ..self }
            }

            pub fn foci(
                &self,
            ) -> std::iter::Zip<
                std::slice::Iter<'_, Vector3>,
                std::slice::Iter<'_, $crate::amp::Amplitude>,
            > {
                self.foci.iter().zip(self.amps.iter())
            }

            pub const fn constraint(&self) -> EmissionConstraint {
                self.constraint
            }
        }
    };
}

pub(crate) trait IntoIntensity {
    fn into_intensity(self) -> f64;
}

impl IntoIntensity for f64 {
    fn into_intensity(self) -> f64 {
        1.
    }
}

impl IntoIntensity for crate::Complex {
    fn into_intensity(self) -> f64 {
        self.abs()
    }
}

pub(crate) fn generate_result<T>(
    geometry: &Geometry,
    q: nalgebra::Matrix<
        T,
        nalgebra::Dyn,
        nalgebra::U1,
        nalgebra::VecStorage<T, nalgebra::Dyn, nalgebra::U1>,
    >,
    max_coefficient: f64,
    constraint: &EmissionConstraint,
    filter: GainFilter,
) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError>
where
    T: Into<Phase> + IntoIntensity + Copy,
{
    match filter {
        GainFilter::All => {
            let num_transducers = geometry
                .iter()
                .scan(0, |state, dev| {
                    let r = *state;
                    *state += dev.num_transducers();
                    Some(r)
                })
                .collect::<Vec<_>>();
            Ok(geometry
                .devices()
                .map(|dev| {
                    (
                        dev.idx(),
                        dev.iter()
                            .zip(q.iter().skip(num_transducers[dev.idx()]))
                            .map(|(_, &q)| {
                                let phase = q.into();
                                let intensity =
                                    constraint.convert(q.into_intensity(), max_coefficient);
                                Drive::new(phase, intensity)
                            })
                            .collect(),
                    )
                })
                .collect())
        }
        GainFilter::Filter(filter) => {
            let num_transducers = geometry
                .iter()
                .scan(0, |state, dev| {
                    let r = *state;
                    *state += filter
                        .get(&dev.idx())
                        .map(|filter| dev.iter().filter(|tr| filter[tr.idx()]).count())
                        .unwrap_or(0);
                    Some(r)
                })
                .collect::<Vec<_>>();
            Ok(geometry
                .devices()
                .map(|dev| {
                    filter.get(&dev.idx()).map_or_else(
                        || (dev.idx(), dev.iter().map(|_| Drive::null()).collect()),
                        |filter| {
                            (
                                dev.idx(),
                                dev.iter()
                                    .filter(|tr| filter[tr.idx()])
                                    .zip(q.iter().skip(num_transducers[dev.idx()]))
                                    .map(|(_, &q)| {
                                        let phase = q.into();
                                        let intensity =
                                            constraint.convert(q.into_intensity(), max_coefficient);
                                        Drive::new(phase, intensity)
                                    })
                                    .collect(),
                            )
                        },
                    )
                })
                .collect())
        }
    }
}
