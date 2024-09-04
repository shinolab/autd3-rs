use std::time::Duration;

use autd3::{
    prelude::{Drive, EmitIntensity, Phase, Silencer, Vector3, AUTD3, ULTRASOUND_PERIOD},
    Controller,
};
use autd3_link_calc::Calc;

#[tokio::test]
async fn gain() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    let rot = autd.geometry()[0].rotation();
    assert!(autd.geometry()[0]
        .iter()
        .zip(autd[0].gain().iter())
        .all(|(tr, (p, r, d))| { tr.position() == p && rot == r && Drive::null() == *d }));

    let expect = Drive::new(Phase::new(0x80), EmitIntensity::new(0x81));
    autd.send(autd3::gain::Uniform::new(expect)).await?;
    assert!(autd[0].gain().iter().all(|(_, _, d)| { expect == *d }));

    Ok(())
}

#[tokio::test]
async fn modulation() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    assert_eq!(
        vec![(Duration::ZERO, 0xFF), (ULTRASOUND_PERIOD, 0xFF)],
        autd[0]
            .modulation()
            .take_while(|(t, _)| *t <= ULTRASOUND_PERIOD)
            .collect::<Vec<_>>()
    );

    let expect = vec![0x80, 0x81];
    autd.send((
        Silencer::disable(),
        autd3::modulation::Custom::new(expect, 2 * ULTRASOUND_PERIOD)?,
    ))
    .await?;
    assert_eq!(
        vec![
            (Duration::ZERO, 0x80),
            (ULTRASOUND_PERIOD, 0x80),
            (2 * ULTRASOUND_PERIOD, 0x81),
            (3 * ULTRASOUND_PERIOD, 0x81)
        ],
        autd[0]
            .modulation()
            .take_while(|(t, _)| *t <= 3 * ULTRASOUND_PERIOD)
            .collect::<Vec<_>>()
    );

    autd.close().await?;

    Ok(())
}
