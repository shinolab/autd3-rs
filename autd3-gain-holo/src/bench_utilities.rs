use criterion::{black_box, AxisScale, BenchmarkId, Criterion, PlotConfiguration};

use autd3_driver::{
    acoustics::directivity::Sphere,
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
                AUTD3::new(Vector3::new(i as f64 * AUTD3::DEVICE_WIDTH, 0., 0.)).into_device(i)
            })
            .collect(),
        FREQ_40K,
    )
}

pub fn gen_foci(n: usize, num_dev: usize) -> impl IntoIterator<Item = (Vector3, Amplitude)> {
    (0..n).map(move |i| {
        (
            Vector3::new(
                black_box(90. + 10. * (2.0 * PI * i as f64 / n as f64).cos()),
                black_box(70. + 10. * (2.0 * PI * i as f64 / n as f64).sin()),
                black_box(150.),
            ),
            5e3 * Pa * num_dev as f64 / n as f64,
        )
    })
}

pub fn sdp_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/sdp_with_{}_devices", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = SDP::new(backend.clone(), gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn naive_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/naive_with_{}_devices", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = Naive::new(backend.clone(), gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn gs_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/gs_with_{}_devices", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = GS::new(backend.clone(), gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn gspat_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/gspat_with_{}_devices", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = GSPAT::new(backend.clone(), gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn lm_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/lm_with_{}_devices", N));
    group
        .sample_size(10)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = LM::new(backend.clone(), gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn greedy_over_foci<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/greedy_with_{}_devices", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    [2].into_iter()
        .chain((2..6).map(|i| i * i))
        .for_each(|size| {
            group.bench_with_input(
                BenchmarkId::new("with_foci_num", size),
                &generate_geometry(N),
                |b, geometry| {
                    let mut tx = TxDatagram::new(size);
                    b.iter(|| {
                        let gen = Greedy::<Sphere>::new(gen_foci(size, N))
                            .operation_generator(geometry)
                            .unwrap();
                        let mut operations = OperationHandler::generate(gen, geometry);
                        OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                    })
                },
            );
        });
    group.finish();
}

pub fn sdp_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/sdp_with_{}_foci", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = SDP::new(backend.clone(), gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

pub fn naive_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/naive_with_{}_foci", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = Naive::new(backend.clone(), gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

pub fn gs_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/gs_with_{}_foci", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = GS::new(backend.clone(), gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

pub fn gspat_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/gspat_with_{}_foci", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = GSPAT::new(backend.clone(), gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

pub fn lm_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/lm_with_{}_foci", N));
    group
        .sample_size(10)
        .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let backend = B::new().unwrap();
    (1..).take(3).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = LM::new(backend.clone(), gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}

pub fn greedy_over_devices<B: LinAlgBackend<Sphere> + 'static, const N: usize>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("autd3-gain-holo/greedy_with_{}_foci", N));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    (1..).take(5).for_each(|size| {
        group.bench_with_input(
            BenchmarkId::new("with_device_num", size * size),
            &generate_geometry(size * size),
            |b, geometry| {
                let mut tx = TxDatagram::new(size);
                b.iter(|| {
                    let gen = Greedy::<Sphere>::new(gen_foci(N, size * size))
                        .operation_generator(geometry)
                        .unwrap();
                    let mut operations = OperationHandler::generate(gen, geometry);
                    OperationHandler::pack(&mut operations, geometry, &mut tx, 0).unwrap();
                })
            },
        );
    });
    group.finish();
}
