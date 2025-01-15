use autd3::{core::link::AsyncLink, prelude::*, r#async::Controller};

#[tokio::test]
async fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Point3::origin())])
        .open(Nop::builder())
        .await?;

    assert!(autd.send(Static::new()).await.is_ok());

    assert!(autd.link_mut().close().await.is_ok());

    assert_eq!(
        Err(AUTDDriverError::LinkClosed),
        autd.send(Static::new()).await
    );

    Ok(())
}
