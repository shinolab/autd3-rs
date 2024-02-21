use std::collections::HashMap;

use autd3_driver::{
    common::Drive, datagram::GainFilter, defined::PI, error::AUTDInternalError, geometry::Geometry,
};
use nalgebra::ComplexField;

use crate::{EmissionConstraint, VectorXc};

#[doc(hidden)]
#[macro_export]
macro_rules! impl_holo {
    ($backend:tt, $t:ty) => {
        impl<$backend> $t
        where
            $backend: $crate::LinAlgBackend,
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

            fn amps_as_slice(&self) -> &[float] {
                unsafe {
                    std::slice::from_raw_parts(self.amps.as_ptr() as *const float, self.amps.len())
                }
            }
        }
    };

    ($t:ty) => {
        impl $t {
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

#[allow(clippy::uninit_vec)]
pub fn generate_result(
    geometry: &Geometry,
    q: VectorXc,
    constraint: &EmissionConstraint,
    filter: GainFilter,
) -> Result<HashMap<usize, Vec<Drive>>, AUTDInternalError> {
    let max_coefficient = q.camax().abs();
    let mut idx = 0;
    match filter {
        GainFilter::All => Ok(geometry
            .devices()
            .map(|dev| {
                (
                    dev.idx(),
                    dev.iter()
                        .map(|_| {
                            let phase =
                                autd3_driver::common::Phase::from_rad(q[idx].argument() + PI);
                            let intensity = constraint.convert(q[idx].abs(), max_coefficient);
                            idx += 1;
                            Drive::new(phase, intensity)
                        })
                        .collect(),
                )
            })
            .collect()),
        GainFilter::Filter(filter) => Ok(geometry
            .devices()
            .map(|dev| {
                filter.get(&dev.idx()).map_or_else(
                    || (dev.idx(), dev.iter().map(|_| Drive::null()).collect()),
                    |filter| {
                        (
                            dev.idx(),
                            dev.iter()
                                .filter(|tr| filter[tr.idx()])
                                .map(|_| {
                                    let phase = autd3_driver::common::Phase::from_rad(
                                        q[idx].argument() + PI,
                                    );
                                    let intensity =
                                        constraint.convert(q[idx].abs(), max_coefficient);
                                    idx += 1;
                                    Drive::new(phase, intensity)
                                })
                                .collect(),
                        )
                    },
                )
            })
            .collect()),
    }
}
