#![allow(clippy::missing_transmute_annotations)]

use autd3::{
    controller::Driver,
    core::gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerMask},
    link::{Audit, AuditOption, audit::version},
    prelude::*,
};

#[test]
fn firmware_v10_by_v12_driver() {
    assert!(
        Controller::<_, firmware::V12>::open_with(
            [AUTD3::default()],
            Audit::<version::V10>::new(AuditOption::default()),
        )
        .is_err()
    );
}

#[test]
fn firmware_v11_by_v12_driver() {
    assert!(
        Controller::<_, firmware::V12>::open_with(
            [AUTD3::default()],
            Audit::<version::V11>::new(AuditOption::default()),
        )
        .is_err()
    );
}

#[test]
fn firmware_v12_by_v12_driver() -> anyhow::Result<()> {
    let mut autd = Controller::<_, firmware::V12>::open_with(
        [AUTD3::default()],
        Audit::<version::V12>::new(AuditOption::default()),
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
        .init(&autd, &autd.environment, &TransducerMask::AllEnabled)?
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
        .init(&autd, &autd.environment, &TransducerMask::AllEnabled)?
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
