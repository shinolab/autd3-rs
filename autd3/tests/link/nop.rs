use autd3::prelude::*;

#[tokio::test]
async fn nop_test() {
    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open_with(Nop::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await
        .unwrap();

    assert_eq!(autd.link.timeout(), std::time::Duration::from_millis(100));

    assert_eq!(autd.send(Static::new()).await, Ok(true));

    assert_eq!(autd.close().await, Ok(true));

    assert_eq!(
        autd.send(Static::new()).await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
    assert_eq!(
        autd.fpga_state().await,
        Err(AUTDError::Internal(AUTDInternalError::LinkClosed))
    );
}
