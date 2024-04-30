use criterion::{black_box, AxisScale, BenchmarkId, Criterion, PlotConfiguration};

use autd3_driver::{
    acoustics::directivity::Sphere,
    autd3_device::AUTD3,
    datagram::{Gain, GainFilter},
    defined::PI,
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
    )
}

pub fn gen_foci(n: usize, num_dev: usize) -> impl Iterator<Item = (Vector3, Amplitude)> {
    (0..n).map(move |i| {
        (
            Vector3::new(
                black_box(90. + 10. * (2.0 * PI * i as f64 / n as f64).cos()),
                black_box(70. + 10. * (2.0 * PI * i as f64 / n as f64).sin()),
                black_box(150.),
            ),
            5e3 * Pascal * num_dev as f64 / n as f64,
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
                    b.iter(|| {
                        SDP::new(backend.clone())
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                    b.iter(|| {
                        Naive::new(backend.clone())
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                    b.iter(|| {
                        GS::new(backend.clone())
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                    b.iter(|| {
                        GSPAT::new(backend.clone())
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                    b.iter(|| {
                        LM::new(backend.clone())
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                    b.iter(|| {
                        Greedy::default()
                            .add_foci_from_iter(gen_foci(size, N))
                            .calc(geometry, GainFilter::All)
                            .unwrap();
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
                b.iter(|| {
                    SDP::new(backend.clone())
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
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
                b.iter(|| {
                    Naive::new(backend.clone())
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
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
                b.iter(|| {
                    GS::new(backend.clone())
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
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
                b.iter(|| {
                    GSPAT::new(backend.clone())
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
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
                b.iter(|| {
                    LM::new(backend.clone())
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
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
                b.iter(|| {
                    Greedy::default()
                        .add_foci_from_iter(gen_foci(N, size * size))
                        .calc(geometry, GainFilter::All)
                        .unwrap();
                })
            },
        );
    });
    group.finish();
}
