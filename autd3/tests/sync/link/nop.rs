use autd3::prelude::*;
use autd3_core::link::Link;

#[test]
fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::open([AUTD3::default()], Nop::new())?;

    assert!(autd.send(Static::default()).is_ok());

    assert!(autd.link_mut().close().is_ok());

    assert_eq!(
        Err(AUTDDriverError::LinkClosed),
        autd.send(Static::default())
    );

    Ok(())
}
