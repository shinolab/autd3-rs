use autd3::{driver::link::Link, prelude::*};

#[tokio::test]
async fn nop_test() -> anyhow::Result<()> {

    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Nop::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await?;

    assert_eq!(std::time::Duration::from_millis(100), autd.link.timeout());

    autd.send(Static::new()).await?;

    autd.close().await?;

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
