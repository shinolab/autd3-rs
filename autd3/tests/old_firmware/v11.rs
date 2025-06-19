#![allow(clippy::missing_transmute_annotations)]

use std::num::NonZeroU16;

use autd3::{
    controller::Driver,
    core::gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerFilter},
    link::{Audit, AuditOption, audit::version},
    prelude::*,
};

#[test]
fn firmware_v10_by_v11_driver() -> anyhow::Result<()> {
    let mut autd = Controller::<_, firmware::V11>::open_with(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )?;

    {
        autd.send((
            Sine {
                freq: 150. * Hz,
                option: Default::default(),
            },
            GainSTM {
                gains: vec![
                    Uniform {
                        intensity: Intensity(0x80),
                        phase: Phase::ZERO,
                    },
                    Uniform {
                        intensity: Intensity(0x81),
                        phase: Phase::ZERO,
                    },
                ],
                config: 1. * Hz,
                option: Default::default(),
            },
        ))?;

        autd.iter().try_for_each(|dev| {
            assert_eq!(
                *Sine {
                    freq: 150. * Hz,
                    option: Default::default(),
                }
                .calc(&firmware::V11.firmware_limits())?,
                autd.link()[dev.idx()]
                    .fpga()
                    .modulation_buffer(unsafe { std::mem::transmute(Segment::S0) })
            );
            let f = Uniform {
                intensity: Intensity(0x80),
                phase: Phase::ZERO,
            }
            .init(autd.geometry(), &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link()[dev.idx()]
                    .fpga()
                    .drives_at(unsafe { std::mem::transmute(Segment::S0) }, 0)
                    .into_iter()
                    .map(|v| unsafe { std::mem::transmute(v) })
                    .collect::<Vec<_>>()
            );
            let f = Uniform {
                intensity: Intensity(0x81),
                phase: Phase::ZERO,
            }
            .init(autd.geometry(), &TransducerFilter::all_enabled())?
            .generate(dev);
            assert_eq!(
                dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
                autd.link()[dev.idx()]
                    .fpga()
                    .drives_at(unsafe { std::mem::transmute(Segment::S0) }, 1)
                    .into_iter()
                    .map(|v| unsafe { std::mem::transmute(v) })
                    .collect::<Vec<_>>()
            );
            anyhow::Ok(())
        })?;
    }

    Ok(())
}

#[test]
#[should_panic]
fn firmware_v10_by_v11_driver_mod_out_of_range() {
    let mut autd = Controller::<_, firmware::V11>::open_with(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )
    .unwrap();

    let _ = autd.send(modulation::Custom {
        buffer: vec![0x00; 65536],
        sampling_config: SamplingConfig::FREQ_4K,
    });
}

#[test]
#[should_panic]
fn firmware_v10_by_v11_driver_focistm_out_of_range() {
    let mut autd = Controller::<_, firmware::V11>::open_with(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )
    .unwrap();

    let _ = autd.send(FociSTM {
        foci: Line {
            start: Point3::origin(),
            end: Point3::origin(),
            num_points: 65536,
            intensity: Intensity::MAX,
        },
        config: SamplingConfig::Divide(NonZeroU16::MAX),
    });
}

#[test]
fn firmware_v11_by_v11_driver() -> anyhow::Result<()> {
    let mut autd = Controller::<_, firmware::V11>::open_with(
        [AUTD3::default()],
        Audit::<version::V11>::new(AuditOption::default()),
    )?;

    autd.send((
        Sine {
            freq: 150. * Hz,
            option: Default::default(),
        },
        GainSTM {
            gains: vec![
                Uniform {
                    intensity: Intensity(0x80),
                    phase: Phase::ZERO,
                },
                Uniform {
                    intensity: Intensity(0x81),
                    phase: Phase::ZERO,
                },
            ],
            config: 1. * Hz,
            option: Default::default(),
        },
    ))?;

    autd.iter().try_for_each(|dev| {
        assert_eq!(
            *Sine {
                freq: 150. * Hz,
                option: Default::default(),
            }
            .calc(&firmware::V11.firmware_limits())?,
            autd.link()[dev.idx()]
                .fpga()
                .modulation_buffer(unsafe { std::mem::transmute(Segment::S0) })
        );
        let f = Uniform {
            intensity: Intensity(0x80),
            phase: Phase::ZERO,
        }
        .init(autd.geometry(), &TransducerFilter::all_enabled())?
        .generate(dev);
        assert_eq!(
            dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
            autd.link()[dev.idx()]
                .fpga()
                .drives_at(unsafe { std::mem::transmute(Segment::S0) }, 0)
                .into_iter()
                .map(|v| unsafe { std::mem::transmute(v) })
                .collect::<Vec<_>>()
        );
        let f = Uniform {
            intensity: Intensity(0x81),
            phase: Phase::ZERO,
        }
        .init(autd.geometry(), &TransducerFilter::all_enabled())?
        .generate(dev);
        assert_eq!(
            dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
            autd.link()[dev.idx()]
                .fpga()
                .drives_at(unsafe { std::mem::transmute(Segment::S0) }, 1)
                .into_iter()
                .map(|v| unsafe { std::mem::transmute(v) })
                .collect::<Vec<_>>()
        );
        anyhow::Ok(())
    })?;

    {
        autd.send(modulation::Custom {
            buffer: vec![0x00; 65536],
            sampling_config: SamplingConfig::FREQ_4K,
        })?;

        autd.send(FociSTM {
            foci: Line {
                start: Point3::origin(),
                end: Point3::origin(),
                num_points: 65536,
                intensity: Intensity::MIN,
            },
            config: SamplingConfig::Divide(NonZeroU16::MAX),
        })?;
    }

    autd.close()?;

    Ok(())
}
