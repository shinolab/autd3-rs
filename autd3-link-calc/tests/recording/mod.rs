use std::time::Duration;

use autd3::{derive::Datagram, prelude::*, Controller};
use autd3_driver::firmware::fpga::FPGA_MAIN_CLK_FREQ;
use autd3_link_calc::{error::CalcError, Calc};

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[tokio::test]
async fn record_pulse_width(#[case] silencer: impl Datagram) -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(silencer).await?;
    autd.start_recording()?;
    autd.send((
        Static::with_intensity(100),
        Uniform::new(EmitIntensity::new(51)),
    ))
    .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send((
        Static::with_intensity(30),
        Uniform::new(EmitIntensity::new(255)),
    ))
    .await?;
    autd.tick(2 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let to_pulse_width = |a, b| {
        let i = (a as usize * b as usize) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };
    assert_eq!(
        vec![
            (Duration::ZERO, to_pulse_width(100, 51)),
            (ULTRASOUND_PERIOD, to_pulse_width(30, 255)),
            (2 * ULTRASOUND_PERIOD, to_pulse_width(30, 255)),
        ],
        record[0][0].pulse_width()
    );

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn record_output_voltage() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.send(PulseWidthEncoder::new(|_dev| {
        |i| match i {
            0x80 => 64,
            0xFF => 128,
            _ => 0,
        }
    }))
    .await?;
    autd.start_recording()?;
    autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0xFF))))
        .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0xFF))))
        .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0x80))))
        .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0x00))))
        .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let v = record[0][0].output_voltage();
    v.iter().enumerate().for_each(|(i, (t, _))| {
        approx::assert_abs_diff_eq!(i as f32 * (1. / FPGA_MAIN_CLK_FREQ.hz() as f32), *t)
    });
    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    assert_eq!(
        [expect_1, expect_2, expect_3, expect_4].concat(),
        v.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
    );

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_already_started() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    assert_eq!(
        Err(CalcError::RecordingAlreadyStarted),
        autd.start_recording()
    );
    let _ = autd.finish_recording()?;

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_not_started() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    assert_eq!(
        Err(CalcError::RecodingNotStarted),
        autd.tick(ULTRASOUND_PERIOD)
    );
    autd.start_recording()?;
    let _ = autd.finish_recording()?;
    assert_eq!(
        CalcError::RecodingNotStarted,
        autd.finish_recording().unwrap_err()
    );

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_invalid_tick() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.tick(ULTRASOUND_PERIOD)?;
    assert_eq!(Err(CalcError::InvalidTick), autd.tick(Duration::ZERO));
    assert_eq!(
        Err(CalcError::InvalidTick),
        autd.tick(ULTRASOUND_PERIOD / 2)
    );
    let _ = autd.finish_recording()?;

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_invalid_op() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::default()).await?;
    autd.start_recording()?;
    autd.tick(ULTRASOUND_PERIOD)?;
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(Silencer::default()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(PulseWidthEncoder::default()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(Clear::new()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send((Null::new(), Clear::new())).await,
    );
    let _ = autd.finish_recording()?;

    autd.close().await?;

    Ok(())
}