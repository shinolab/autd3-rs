use anyhow::Result;

use autd3::{derive::*, driver::firmware_version::FirmwareInfo, prelude::*};
use autd3_link_soem::{Status, SOEM};

fn print_msg_and_wait_for_key(msg: &str) {
    print!("{}", msg);
    println!(" Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    print_msg_and_wait_for_key(
        "Make sure you have two devices connected that have the latest firmware.\nAlso check that an oscilloscope is connected to GPIO[0] and GPIO[1] of each device.\nAnd check if outputs of GPIO[0] pins are NOT synchronized each other.",
    );

    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .add_device(AUTD3::new(Vector3::zeros()))
        .open(
            SOEM::builder().with_err_handler(|slave, status| match status {
                Status::Error(msg) => eprintln!("Error [{}]: {}", slave, msg),
                Status::Lost(msg) => {
                    eprintln!("Lost [{}]: {}", slave, msg);
                    std::process::exit(-1);
                }
                Status::StateChanged(msg) => eprintln!("StateChanged [{}]: {}", slave, msg),
            }),
        )
        .await?;

    print_msg_and_wait_for_key("Check if outputs of GPIO[0] pins are now synchronized.");

    let firmware_infos = autd.firmware_infos().await?;
    assert_eq!(2, firmware_infos.len());
    firmware_infos.iter().for_each(|firm_info| {
        assert_eq!(
            FirmwareInfo::LATEST_VERSION_NUM_MAJOR,
            firm_info.fpga_version_number_major()
        );
        assert_eq!(
            FirmwareInfo::LATEST_VERSION_NUM_MINOR,
            firm_info.fpga_version_number_minor()
        );
        assert_eq!(
            FirmwareInfo::LATEST_VERSION_NUM_MAJOR,
            firm_info.cpu_version_number_major()
        );
        assert_eq!(
            FirmwareInfo::LATEST_VERSION_NUM_MINOR,
            firm_info.cpu_version_number_minor()
        );
    });

    autd.send(ConfigureReadsFPGAState::new(|_| true)).await?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    // Gain Chcek
    {
        autd.send((
            Sine::new(150.0),
            Focus::new(autd.geometry.center() + 150. * Vector3::z()),
        ))
        .await?;
        print_msg_and_wait_for_key(
            "Check that the focal points are generated 150mm directly above the center of each device by your hands."
        );

        autd.send(Null::new().with_segment(Segment::S1, true))
            .await?;
        print_msg_and_wait_for_key("Check that the focal points have disappeared.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S1), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(ChangeGainSegment::new(Segment::S0)).await?;
        print_msg_and_wait_for_key("Check that the focal points are presented again.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(Null::new().with_segment(Segment::S1, false))
            .await?;
        print_msg_and_wait_for_key("Check that the focal points are still presented.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(ChangeGainSegment::new(Segment::S1)).await?;
        print_msg_and_wait_for_key("Check that the focal points have disappeared.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S1), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S1)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S1)).await
        );
    }

    // Modulation check
    {
        autd.send((
            Sine::new(150.0),
            Focus::new(autd.geometry.center() + 150. * Vector3::z()),
        ))
        .await?;
        print_msg_and_wait_for_key(
            "Check that the focal points are generated 150mm directly above the center of each device by your hands."
        );
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(Static::new().with_segment(Segment::S1, true))
            .await?;
        print_msg_and_wait_for_key("Check that the AM modulation is no longer applied.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S1, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(ChangeModulationSegment::new(Segment::S0)).await?;
        print_msg_and_wait_for_key("Check that the AM modulation has been applied again.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(Static::with_intensity(0).with_segment(Segment::S1, false))
            .await?;
        print_msg_and_wait_for_key("Check that the focal points are still presented.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(ChangeModulationSegment::new(Segment::S1)).await?;
        print_msg_and_wait_for_key("Check that the focal points have disappeared.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S1, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        #[derive(Modulation, Clone, Copy)]
        pub struct Sawtooth {
            config: SamplingConfiguration,
            loop_behavior: LoopBehavior,
            reverse: bool,
        }

        impl Sawtooth {
            pub fn new() -> Self {
                Self {
                    config: SamplingConfiguration::from_frequency(256.).unwrap(),
                    loop_behavior: LoopBehavior::once(),
                    reverse: false,
                }
            }

            pub fn reverse() -> Self {
                Self {
                    config: SamplingConfiguration::from_frequency(256.).unwrap(),
                    loop_behavior: LoopBehavior::once(),
                    reverse: true,
                }
            }
        }

        impl Modulation for Sawtooth {
            fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
                let mut res = (0..=255u8)
                    .map(|i| EmitIntensity::new(i))
                    .collect::<Vec<_>>();
                if self.reverse {
                    res.reverse();
                }
                Ok(res)
            }
        }

        autd.send(Sawtooth::new().with_segment(Segment::S0, true))
            .await?;
        print_msg_and_wait_for_key(
            "Check that the AM modulation is applied with a sawtooth pattern.",
        );
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });

        autd.send(Sawtooth::reverse().with_segment(Segment::S1, true))
            .await?;
        print_msg_and_wait_for_key(
            "Check that the AM modulation is applied with a reversed sawtooth pattern.",
        );
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S1, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });
    }

    // FocusSTM check
    {
        autd.send(Static::new()).await?;

        let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
        let point_num = 200;
        let radius = 30.0 * MILLIMETER;
        let gen_foci = || {
            (0..point_num).map(|i| {
                let theta = 2.0 * PI * i as float / point_num as float;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                ControlPoint::new(center + p).with_intensity(0xFF)
            })
        };

        let stm = FocusSTM::from_freq(0.5).add_foci_from_iter(gen_foci())?;
        autd.send(stm).await?;
        print_msg_and_wait_for_key(
            "Check that the focal points are moving at a frequency of 0.5 Hz over a circumference of 30 mm radius by your hands."
        );
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });

        let stm = FocusSTM::from_freq(1.).add_foci_from_iter(gen_foci())?;
        autd.send(stm.with_segment(Segment::S1, true)).await?;
        print_msg_and_wait_for_key("Check that the frequency is now 1 Hz.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S1), state.current_stm_segment());
        });

        autd.send(ChangeFocusSTMSegment::new(Segment::S0)).await?;
        print_msg_and_wait_for_key("Check that the frequency returned to 0.5 Hz.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });

        let mut foci = gen_foci().rev().collect::<Vec<_>>();
        foci[point_num - 1] = foci[point_num - 1].with_intensity(0x00);
        let stm = FocusSTM::from_freq(0.5)
            .with_loop_behavior(LoopBehavior::once())
            .add_foci_from_iter(foci)?
            .with_segment(Segment::S1, false);
        autd.send(stm).await?;
        print_msg_and_wait_for_key("Check that the nothing has chenged. Then, continue if the focal point is on the left size of device and check that the focus movement direction reverses when the focus comes to the right edge and stops after a cycle.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });
        autd.send(ChangeFocusSTMSegment::new(Segment::S1)).await?;
        print_msg_and_wait_for_key("");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S1), state.current_stm_segment());
        });

        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S1)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSegment::new(Segment::S1)).await
        );
    }

    // GainSTM check
    {
        autd.send(Static::new()).await?;

        let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
        let point_num = 200;
        let radius = 30.0 * MILLIMETER;
        let gen_foci = || {
            (0..point_num).map(|i| {
                let theta = 2.0 * PI * i as float / point_num as float;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                Focus::new(center + p).with_intensity(0xFF)
            })
        };

        let stm = GainSTM::from_freq(0.5).add_gains_from_iter(gen_foci())?;
        autd.send(stm).await?;
        print_msg_and_wait_for_key(
            "Check that the focal points are moving at a frequency of 0.5 Hz over a circumference of 30 mm radius by your hands."
        );
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });

        let stm = GainSTM::from_freq(1.).add_gains_from_iter(gen_foci())?;
        autd.send(stm.with_segment(Segment::S1, true)).await?;
        print_msg_and_wait_for_key("Check that the frequency is now 1 Hz.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S1), state.current_stm_segment());
        });

        autd.send(ChangeGainSTMSegment::new(Segment::S0)).await?;
        print_msg_and_wait_for_key("Check that the frequency returned to 0.5 Hz.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });

        let mut foci = gen_foci().rev().collect::<Vec<_>>();
        foci[point_num - 1] = Focus::new(*foci[point_num - 1].pos()).with_intensity(0x00);
        let stm = GainSTM::from_freq(0.5)
            .with_loop_behavior(LoopBehavior::once())
            .add_gains_from_iter(foci)?
            .with_segment(Segment::S1, false);
        autd.send(stm).await?;
        print_msg_and_wait_for_key("Check that the nothing has chenged. Then, continue if the focal point is on the left size of device and check that the focus movement direction reverses when the focus comes to the right edge and stops after a cycle.");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S0), state.current_stm_segment());
        });
        autd.send(ChangeGainSTMSegment::new(Segment::S1)).await?;
        print_msg_and_wait_for_key("");
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(None, state.current_gain_segment());
            assert_eq!(Some(Segment::S1), state.current_stm_segment());
        });

        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S1)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSegment::new(Segment::S1)).await
        );
    }

    // clear
    {
        autd.send(Clear::new()).await?;
        autd.send(ConfigureReadsFPGAState::new(|_| true)).await?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        autd.fpga_state().await?.iter().for_each(|state| {
            assert!(state.is_some());
            let state = state.unwrap();
            assert_eq!(Segment::S0, state.current_mod_segment());
            assert_eq!(Some(Segment::S0), state.current_gain_segment());
            assert_eq!(None, state.current_stm_segment());
        });
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeFocusSTMSegment::new(Segment::S1)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S0)).await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSegmentTransition
            )),
            autd.send(ChangeGainSTMSegment::new(Segment::S1)).await
        );
    }

    // Phase filter
    {
        autd.send(ConfigurePhaseFilter::additive(|dev, tr| {
            tr.align_phase_at(
                dev.center() + Vector3::new(0.0, 0.0, 150.0 * MILLIMETER),
                dev.sound_speed,
            )
        }))
        .await?;
        autd.send((
            Sine::new(150.0),
            Uniform::new(0xFF).with_phase(Phase::new(0)),
        ))
        .await?;
        print_msg_and_wait_for_key(
            "Check that the focal points are generated 150mm directly above the center of each device by your hands."
        );

        autd.send(ConfigurePhaseFilter::additive(|_dev, _tr| Phase::new(0)))
            .await?;
        autd.send(Static::new()).await?;
    }

    // Debug output index
    {
        autd.send(TransducerTest::new(|dev, tr| match (dev.idx(), tr.idx()) {
            (0, 0) => Some(Drive::new(Phase::new(0), EmitIntensity::new(0xFF))),
            (0, 248) => Some(Drive::new(Phase::new(0x80), EmitIntensity::new(0x80))),
            (1, 0) => Some(Drive::new(Phase::new(0x80), EmitIntensity::new(0xFF))),
            (1, 248) => Some(Drive::new(Phase::new(0), EmitIntensity::new(0x80))),
            _ => None,
        }))
        .await?;
        print_msg_and_wait_for_key("Check that there are no outputs of GPIO[2] pins.");

        autd.send(ConfigureDebugOutputIdx::new(|dev| match dev.idx() {
            0 => Some(&dev[0]),
            1 => Some(&dev[0]),
            _ => None,
        }))
        .await?;
        print_msg_and_wait_for_key("Check that a 40kHz square wave with a duty ratio of 50% are output to the GPIO[2] pins and that the phase is shifted by half a cycle.");

        autd.send(ConfigureDebugOutputIdx::new(|dev| match dev.idx() {
            0 => Some(&dev[248]),
            1 => Some(&dev[248]),
            _ => None,
        }))
        .await?;
        print_msg_and_wait_for_key("Check that a 40kHz square wave with a duty ratio of about 17% are output to the GPIO[2] pins and that the phase is shifted by half a cycle.");

        autd.send(ConfigureDebugOutputIdx::new(|dev| match dev.idx() {
            0 => Some(&dev[0]),
            1 => Some(&dev[248]),
            _ => None,
        }))
        .await?;
        print_msg_and_wait_for_key("Check that a 40 kHz square wave are output on the GPIO[2] pins and that their phase are aligned.");

        autd.send(ConfigureDebugOutputIdx::new(|dev| match dev.idx() {
            0 => Some(&dev[1]),
            1 => Some(&dev[2]),
            _ => None,
        }))
        .await?;
        print_msg_and_wait_for_key("Check that there are no outputs of GPIO[2] pins.");
    }

    autd.close().await?;

    println!("Ok!");
    Ok(())
}
