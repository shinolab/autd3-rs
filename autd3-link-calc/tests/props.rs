use autd3::{
    prelude::{Drive, EmitIntensity, Phase, Vector3, AUTD3},
    Controller,
};
use autd3_link_calc::Calc;

#[tokio::test]
async fn gain() -> anyhow::Result<()> {
    let mut autd = Controller::builder([
        AUTD3::new(Vector3::zeros()),
        AUTD3::new(Vector3::new(10., 20., 30.)),
    ])
    .open(Calc::builder())
    .await?;

    let rot = autd.geometry()[0].rotation();
    assert!(autd
        .geometry()
        .iter()
        .flat_map(|dev| dev.iter())
        .zip(autd.gain().iter())
        .all(|(tr, (p, r, d))| { tr.position() == p && rot == r && Drive::null() == *d }));

    let expect = Drive::new(Phase::new(0x80), EmitIntensity::new(0x81));
    autd.send(autd3::gain::Uniform::new(expect)).await?;
    assert!(autd.gain().iter().all(|(_, _, d)| { expect == *d }));

    autd.close().await?;

    Ok(())
}
