#![allow(clippy::missing_transmute_annotations)]

use autd3::{
    controller::Driver,
    core::gain::{Gain, GainCalculator, GainCalculatorGenerator, TransducerMask},
    link::{Audit, AuditOption, audit::version},
    prelude::*,
};

#[test]
fn firmware_v10_by_v10_driver() -> Result<(), Box<dyn std::error::Error>> {
    let mut autd = Controller::<_, firmware::V10>::open_with(
        [AUTD3::default()],
        Audit::<version::V10>::new(AuditOption::default()),
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
        Result::<(), Box<dyn std::error::Error>>::Ok(())
    })?;

    autd.close()?;

    Ok(())
}
