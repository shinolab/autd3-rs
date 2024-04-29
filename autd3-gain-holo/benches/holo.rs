#[cfg(feature = "bench-utilities")]
criterion::criterion_group!(
    benches,
    autd3_gain_holo::bench_utilities::sdp_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::naive_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::gs_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::gspat_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::lm_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::greedy_over_foci::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::sdp_over_devices::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::naive_over_devices::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::gs_over_devices::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::gspat_over_devices::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::lm_over_devices::<autd3_gain_holo::NalgebraBackend, 4>,
    autd3_gain_holo::bench_utilities::greedy_over_devices::<autd3_gain_holo::NalgebraBackend, 4>
);
#[cfg(feature = "bench-utilities")]
criterion::criterion_main!(benches);

#[cfg(not(feature = "bench-utilities"))]
fn main() {
    panic!("This benchmark requires `bench-utilities` feature");
}
