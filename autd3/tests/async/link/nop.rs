use autd3::{core::link::AsyncLink, prelude::*, r#async::Controller};

#[tokio::test]
async fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::open([AUTD3::default()], Nop::new()).await?;

    assert!(autd.send(Static::default()).await.is_ok());

    assert!(autd.link_mut().close().await.is_ok());

    assert_eq!(
        Err(AUTDDriverError::LinkClosed),
        autd.send(Static::default()).await
    );

    Ok(())
}
