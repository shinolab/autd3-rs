use autd3::prelude::*;
use autd3_core::link::Link;

#[test]
fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Point3::origin())]).open(Nop::builder())?;

    assert!(autd.send(Static::new()).is_ok());

    assert!(autd.link_mut().close().is_ok());

    assert_eq!(Err(AUTDDriverError::LinkClosed), autd.send(Static::new()));

    Ok(())
}
