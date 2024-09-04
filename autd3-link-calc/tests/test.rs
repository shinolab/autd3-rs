mod recording;

use autd3::{
    prelude::{Drive, Vector3, AUTD3},
    Controller,
};
use autd3_link_calc::Calc;

#[tokio::test]
async fn update() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.geometry_mut()[0].translate(Vector3::new(10., 20., 30.));
    let rot = autd.geometry()[0].rotation();
    assert!(autd.geometry()[0]
        .iter()
        .zip(autd[0].gain().iter())
        .all(|(tr, (p, r, d))| { tr.position() != p && rot == r && Drive::null() == *d }));
    autd.send(autd3::gain::Null::new()).await?;
    let rot = autd.geometry()[0].rotation();
    assert!(autd.geometry()[0]
        .iter()
        .zip(autd[0].gain().iter())
        .all(|(tr, (p, r, d))| { tr.position() == p && rot == r && Drive::null() == *d }));

    autd.close().await?;

    Ok(())
}
