/*
 * File: holo.rs
 * Project: benches
 * Created Date: 31/07/2023
 * Author: Shun Suzuki
 * -----
 * Last Modified: 17/01/2024
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2023 Shun Suzuki. All rights reserved.
 *
 */

#[cfg(feature = "bench-utilities")]
criterion::criterion_group!(
    benches,
    autd3_gain_holo::bench_utilities::foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::devices::<autd3_gain_holo::NalgebraBackend, 2>
);
#[cfg(feature = "bench-utilities")]
criterion::criterion_main!(benches);

#[cfg(not(feature = "bench-utilities"))]
fn main() {
    panic!("This benchmark requires `bench-utilities` feature");
}
