use criterion::{black_box, AxisScale, Criterion, PlotConfiguration};

use autd3_driver::{
    acoustics::directivity::Directivity,
    autd3_device::AUTD3,
    defined::{FREQ_40K, PI},
    derive::Datagram,
    firmware::{cpu::TxDatagram, operation::OperationHandler},
    geometry::{Geometry, IntoDevice, Vector3},
};

use crate::*;

pub fn generate_geometry(size: usize) -> Geometry {
    Geometry::new(
        (0..size)
            .map(move |i| {
                AUTD3::new(Vector3::new(i as f32 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
        FREQ_40K,
    )
}

pub fn gen_foci(n: usize, num_dev: usize) -> impl IntoIterator<Item = (Vector3, Amplitude)> {
    (0..n).map(move |i| {
        (
            Vector3::new(
                black_box(90. + 10. * (2.0 * PI * i as f32 / n as f32).cos()),
                black_box(70. + 10. * (2.0 * PI * i as f32 / n as f32).sin()),
                black_box(150.),
            ),
            5e3 * Pa * num_dev as f32 / n as f32,
        )
    })
}

pub fn sdp<
    D: Directivity + 'static,
    B: LinAlgBackend<D> + 'static,
    const NUM_DEV: usize,
    const NUM_FOCI: usize,
>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();

    group.bench_with_input(
        format!("SDP/{}-devices_{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= SDP::new(backend.clone(), gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}

pub fn naive<
    D: Directivity + 'static,
    B: LinAlgBackend<D> + 'static,
    const NUM_DEV: usize,
    const NUM_FOCI: usize,
>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();

    group.bench_with_input(
        format!("Naive/{}_devices-{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= Naive::new(backend.clone(), gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}

pub fn gs<
    D: Directivity + 'static,
    B: LinAlgBackend<D> + 'static,
    const NUM_DEV: usize,
    const NUM_FOCI: usize,
>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();

    group.bench_with_input(
        format!("GS/{}_devices-{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= GS::new(backend.clone(), gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}

pub fn gspat<
    D: Directivity + 'static,
    B: LinAlgBackend<D> + 'static,
    const NUM_DEV: usize,
    const NUM_FOCI: usize,
>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();

    group.bench_with_input(
        format!("GSPAT/{}_devices-{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= GSPAT::new(backend.clone(), gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}

pub fn lm<
    D: Directivity + 'static,
    B: LinAlgBackend<D> + 'static,
    const NUM_DEV: usize,
    const NUM_FOCI: usize,
>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();

    group.bench_with_input(
        format!("LM/{}_devices-{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= LM::new(backend.clone(), gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}

pub fn greedy<D: Directivity + 'static, const NUM_DEV: usize, const NUM_FOCI: usize>(
    c: &mut Criterion,
) {
    let mut group = c.benchmark_group("autd3-gain-holo");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    group.bench_with_input(
        format!("Greedy/{}_devices-{}-foci", NUM_DEV, NUM_FOCI),
        &generate_geometry(NUM_DEV),
        |b, geometry| {
            let mut tx = TxDatagram::new(NUM_DEV);
            b.iter(|| {
                let generator= Greedy::<D>::new(gen_foci(NUM_FOCI, NUM_DEV))
                    .operation_generator(geometry)
                    .unwrap();
                let mut operations = OperationHandler::generate(gen, geometry);
                OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
            })
        },
    );
    group.finish();
}
