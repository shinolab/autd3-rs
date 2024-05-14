use std::time::Duration;

use autd3_driver::{
    datagram::Silencer,
    derive::*,
    error::AUTDInternalError,
    ethercat::{DcSysTime, ECAT_DC_SYS_TIME_BASE},
    firmware::{
        cpu::TxDatagram,
        fpga::{
            GPIOIn, TransitionMode, SAMPLING_FREQ_DIV_MAX, SAMPLING_FREQ_DIV_MIN,
            SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT,
        },
        operation::{ModulationOp, ModulationSwapSegmentOp, SwapSegmentOperation},
    },
};
use autd3_firmware_emulator::{cpu::params::SYS_TIME_TRANSITION_MARGIN, CPUEmulator};

use time::OffsetDateTime;

use rand::*;

use crate::{create_geometry, send};

#[derive(Modulation)]
pub struct TestModulation {
    pub buf: Vec<u8>,
    pub config: SamplingConfig,
    pub loop_behavior: LoopBehavior,
}

impl Modulation for TestModulation {
    fn calc(&self, _: &Geometry) -> Result<Vec<u8>, AUTDInternalError> {
        Ok(self.buf.clone())
    }
}

#[test]
fn send_mod() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();

    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let m: Vec<_> = (0..32768).map(|_| rng.gen()).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::infinite();
        let transition_mode = TransitionMode::Immediate;
        let mut op = ModulationOp::new(
            TestModulation {
                buf: m.clone(),
                config: SamplingConfig::DivisionRaw(freq_div),
                loop_behavior,
            },
            Segment::S0,
            Some(transition_mode),
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert_eq!(Segment::S0, cpu.fpga().req_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S0));
        assert_eq!(freq_div, cpu.fpga().modulation_freq_division(Segment::S0));
        assert_eq!(
            loop_behavior,
            cpu.fpga().modulation_loop_behavior(Segment::S0)
        );
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
        assert_eq!(m, cpu.fpga().modulation(Segment::S0));
    }

    {
        let m: Vec<_> = (0..2).map(|_| rng.gen()).collect();
        let freq_div = rng.gen_range(
            SAMPLING_FREQ_DIV_MIN
                * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32
                ..=SAMPLING_FREQ_DIV_MAX,
        );
        let loop_behavior = LoopBehavior::once();
        let mut op = ModulationOp::new(
            TestModulation {
                buf: m.clone(),
                config: SamplingConfig::DivisionRaw(freq_div),
                loop_behavior,
            },
            Segment::S1,
            None,
        );

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert_eq!(Segment::S0, cpu.fpga().req_mod_segment());
        assert_eq!(m.len(), cpu.fpga().modulation_cycle(Segment::S1));
        assert_eq!(freq_div, cpu.fpga().modulation_freq_division(Segment::S1));
        assert_eq!(
            loop_behavior,
            cpu.fpga().modulation_loop_behavior(Segment::S1)
        );
        assert_eq!(TransitionMode::Immediate, cpu.fpga().mod_transition_mode());
        assert_eq!(m, cpu.fpga().modulation(Segment::S1));
    }

    {
        let mut op = ModulationSwapSegmentOp::new(Segment::S1, TransitionMode::SyncIdx);

        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        assert_eq!(Segment::S1, cpu.fpga().req_mod_segment());
        assert_eq!(TransitionMode::SyncIdx, cpu.fpga().mod_transition_mode());
    }

    {
        [
            (Segment::S0, GPIOIn::I0),
            (Segment::S1, GPIOIn::I1),
            (Segment::S0, GPIOIn::I2),
            (Segment::S1, GPIOIn::I3),
        ]
        .into_iter()
        .for_each(|(segment, gpio)| {
            let transition_mode = TransitionMode::GPIO(gpio);
            let mut op = ModulationOp::new(
                TestModulation {
                    buf: (0..2).map(|_| u8::MAX).collect(),
                    config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                    loop_behavior: LoopBehavior::once(),
                },
                segment,
                Some(transition_mode),
            );
            assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
            assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
        });
    }

    {
        let transition_mode = TransitionMode::Ext;
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(transition_mode),
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    Ok(())
}

#[test]
fn mod_freq_div_too_small() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    {
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MIN),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(TransitionMode::Immediate),
        );

        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        )
    }

    {
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(TransitionMode::Immediate),
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let (mut op, _) = Silencer::fixed_completion_steps(
            SILENCER_STEPS_INTENSITY_DEFAULT,
            SILENCER_STEPS_PHASE_DEFAULT,
        )?
        .operation();
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(
                    SAMPLING_FREQ_DIV_MIN
                        * SILENCER_STEPS_INTENSITY_DEFAULT.max(SILENCER_STEPS_PHASE_DEFAULT) as u32,
                ),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S1,
            None,
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let (mut op, _) = Silencer::fixed_completion_steps(
            SILENCER_STEPS_PHASE_DEFAULT * 2,
            SILENCER_STEPS_PHASE_DEFAULT,
        )?
        .operation();
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = ModulationSwapSegmentOp::new(Segment::S1, TransitionMode::Immediate);
        assert_eq!(
            Err(AUTDInternalError::InvalidSilencerSettings),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[test]
fn send_mod_invalid_transition_mode() -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    // segment 0 to 0
    {
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S0,
            Some(TransitionMode::SyncIdx),
        );
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    // segment 0 to 1 immidiate
    {
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                loop_behavior: LoopBehavior::once(),
            },
            Segment::S1,
            Some(TransitionMode::Immediate),
        );
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    // Infinite but SyncIdx
    {
        let mut op = ModulationOp::new(
            TestModulation {
                buf: (0..2).map(|_| u8::MAX).collect(),
                config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
                loop_behavior: LoopBehavior::infinite(),
            },
            Segment::S1,
            None,
        );
        assert_eq!(Ok(()), send(&mut cpu, &mut op, &geometry, &mut tx));

        let mut op = ModulationSwapSegmentOp::new(Segment::S1, TransitionMode::SyncIdx);
        assert_eq!(
            Err(AUTDInternalError::InvalidTransitionMode),
            send(&mut cpu, &mut op, &geometry, &mut tx)
        );
    }

    Ok(())
}

#[rstest::rstest]
#[test]
#[case(Ok(()), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE, ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN-autd3_driver::ethercat::EC_CYCLE_TIME_BASE_NANO_SEC))]
#[case(Err(AUTDInternalError::MissTransitionTime), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(1), ECAT_DC_SYS_TIME_BASE + Duration::from_nanos(SYS_TIME_TRANSITION_MARGIN))]
fn test_miss_transition_time(
    #[case] expect: Result<(), AUTDInternalError>,
    #[case] systime: OffsetDateTime,
    #[case] transition_time: OffsetDateTime,
) -> anyhow::Result<()> {
    let geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = TxDatagram::new(geometry.num_devices());

    let transition_mode = TransitionMode::SysTime(DcSysTime::from_utc(transition_time).unwrap());
    let mut op = ModulationOp::new(
        TestModulation {
            buf: (0..2).map(|_| u8::MAX).collect(),
            config: SamplingConfig::DivisionRaw(SAMPLING_FREQ_DIV_MAX),
            loop_behavior: LoopBehavior::once(),
        },
        Segment::S1,
        Some(transition_mode),
    );

    cpu.update_with_sys_time(DcSysTime::from_utc(systime).unwrap());
    assert_eq!(expect, send(&mut cpu, &mut op, &geometry, &mut tx));
    if expect.is_ok() {
        assert_eq!(transition_mode, cpu.fpga().mod_transition_mode());
    }

    Ok(())
}
