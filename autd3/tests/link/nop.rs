use autd3::prelude::*;

#[tokio::test]
async fn nop_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(Nop::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await?;

    assert_eq!(std::time::Duration::from_millis(100), autd.link.timeout());

    assert!(autd.send(Static::new()).await?);

    assert!(autd.close().await?);

    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed)),
        autd.send(Static::new()).await
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed)),
        autd.fpga_state().await
    );

    Ok(())
}
