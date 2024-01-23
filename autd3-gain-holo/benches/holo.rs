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
