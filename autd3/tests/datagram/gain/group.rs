use autd3::{link::Audit, prelude::*};

#[tokio::test]
async fn only_for_enabled() -> anyhow::Result<()> {
    let mut autd =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .open(Audit::builder())
            .await?;

    let check = std::sync::Arc::new(std::sync::Mutex::new(vec![
        false;
        autd.geometry().num_devices()
    ]));

    autd.geometry_mut()[0].enable = false;

    autd.send(
        Group::new(|dev| {
            check.lock().unwrap()[dev.idx()] = true;
            move |_| Some(0)
        })
        .set(
            0,
            Uniform::new(Drive::new(Phase::new(0x90), EmitIntensity::new(0x80))),
        ),
    )
    .await?;

    assert!(!check.lock().unwrap()[0]);
    assert!(check.lock().unwrap()[1]);

    assert!(autd[0]
        .fpga()
        .drives_at(Segment::S0, 0)
        .into_iter()
        .all(|d| Drive::null() == d));
    assert!(autd[1]
        .fpga()
        .drives_at(Segment::S0, 0)
        .into_iter()
        .all(|d| Drive::new(Phase::new(0x90), EmitIntensity::new(0x80)) == d));

    Ok(())
}
