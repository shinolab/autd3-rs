use autd3::{driver::link::Link, prelude::*};

#[tokio::test]
async fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Nop::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await?;

    assert_eq!(std::time::Duration::from_millis(100), autd.link().timeout());

    assert!(autd.send(Static::new()).await.is_ok());

    assert!(autd.link_mut().close().await.is_ok());

    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed)),
        autd.send(Static::new()).await
    );

    Ok(())
}
