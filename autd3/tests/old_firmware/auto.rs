#![allow(clippy::missing_transmute_annotations)]

use std::num::NonZeroU16;

use autd3::{
    controller::Driver,
    core::gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerFilter},
    firmware::Version,
    link::{Audit, AuditOption, audit::version},
    prelude::*,
};
use autd3_driver::datagram::Nop;

#[test]
fn firmware_v10_by_auto_driver() -> anyhow::Result<()> {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )?;

    assert_eq!(Version::V10, autd.driver().version());

    assert_eq!(
        "0: CPU = v10.0.1, FPGA = v10.0.1 [Emulator]",
        autd.firmware_version()?[0].to_string()
    );

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

    autd.close()?;

    Ok(())
}

#[test]
fn firmware_v10_by_auto_driver_mod_out_of_range() {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )
    .unwrap();

    assert_eq!(
        Err(AUTDDriverError::ModulationSizeOutOfRange(
            65536,
            firmware::V10.firmware_limits()
        )),
        autd.send(modulation::Custom {
            buffer: vec![0x00; 65536],
            sampling_config: SamplingConfig::FREQ_4K,
        })
    );
}

#[test]
fn firmware_v10_by_auto_driver_focistm_out_of_range() {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
    )
    .unwrap();

    assert_eq!(
        Err(AUTDDriverError::FociSTMTotalSizeOutOfRange(
            65536,
            firmware::V10.firmware_limits()
        )),
        autd.send(FociSTM {
            foci: Line {
                start: Point3::origin(),
                end: Point3::origin(),
                num_points: 65536,
                intensity: Intensity::MAX,
            },
            config: SamplingConfig::Divide(NonZeroU16::MAX),
        })
    );
}

#[test]
fn firmware_v11_by_auto_driver() -> anyhow::Result<()> {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V11>::new(AuditOption::default()),
    )?;

    assert_eq!(Version::V11, autd.driver().version());

    assert_eq!(
        "0: CPU = v11.0.0, FPGA = v11.0.0 [Emulator]",
        autd.firmware_version()?[0].to_string()
    );

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

    autd.close()?;

    Ok(())
}

#[test]
fn firmware_v10_by_auto_driver_nop() {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V11>::new(AuditOption::default()),
    )
    .unwrap();

    assert_eq!(Err(AUTDDriverError::UnsupportedOperation), autd.send(Nop));
}

#[test]
fn firmware_v11_by_auto_driver_nop() {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V11>::new(AuditOption::default()),
    )
    .unwrap();

    assert_eq!(Err(AUTDDriverError::UnsupportedOperation), autd.send(Nop));
}

#[test]
fn firmware_v12_by_auto_driver() -> anyhow::Result<()> {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V12>::new(AuditOption::default()),
    )?;

    assert_eq!(Version::V12, autd.driver().version());

    assert_eq!(
        "0: CPU = v12.0.0, FPGA = v12.0.0 [Emulator]",
        autd.firmware_version()?[0].to_string()
    );

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
            .calc(&firmware::V12.firmware_limits())?,
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

    autd.close()?;

    Ok(())
}

#[test]
fn firmware_v12_by_auto_driver_output_mask() {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V12>::new(AuditOption::default()),
    )
    .unwrap();

    assert_eq!(
        Err(AUTDDriverError::UnsupportedOperation),
        autd.send(OutputMask::new(|_| |_| true, Segment::S0))
    );
}

#[test]
fn firmware_v12_1_by_auto_driver() -> anyhow::Result<()> {
    let mut autd = Controller::open(
        [AUTD3::default()],
        Audit::<version::V12_1>::new(AuditOption::default()),
    )?;

    assert_eq!(Version::V12_1, autd.driver().version());

    assert_eq!(
        "0: CPU = v12.1.0, FPGA = v12.1.0 [Emulator]",
        autd.firmware_version()?[0].to_string()
    );

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
            .calc(&firmware::V12.firmware_limits())?,
            autd.link()[dev.idx()].fpga().modulation_buffer(Segment::S0)
        );
        let f = Uniform {
            intensity: Intensity(0x80),
            phase: Phase::ZERO,
        }
        .init(autd.geometry(), &TransducerFilter::all_enabled())?
        .generate(dev);
        assert_eq!(
            dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
            autd.link()[dev.idx()].fpga().drives_at(Segment::S0, 0)
        );
        let f = Uniform {
            intensity: Intensity(0x81),
            phase: Phase::ZERO,
        }
        .init(autd.geometry(), &TransducerFilter::all_enabled())?
        .generate(dev);
        assert_eq!(
            dev.iter().map(|tr| f.calc(tr)).collect::<Vec<_>>(),
            autd.link()[dev.idx()].fpga().drives_at(Segment::S0, 1)
        );
        anyhow::Ok(())
    })?;

    autd.close()?;

    Ok(())
}
