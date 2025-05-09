use autd3::{
    r#async::Controller,
    gain::Null,
    link::{Audit, AuditOption},
    prelude::{AUTD3, AUTDDriverError},
};

mod link;

#[tokio::test]
async fn initial_msg_id() -> anyhow::Result<()> {
    let cnt = Controller::open(
        [AUTD3::default()],
        Audit::new(AuditOption {
            initial_msg_id: Some(0x01),
            initial_phase_corr: Some(0xFF),
            ..Default::default()
        }),
    )
    .await?;

    assert!(
        cnt.link()[0]
            .fpga()
            .phase_correction()
            .iter()
            .all(|v| v.0 == 0x00)
    );

    Ok(())
}

#[tokio::test]
async fn test_retry_with_disabled_device() -> anyhow::Result<()> {
    let mut cnt = Controller::open([AUTD3::default(); 2], Audit::new(Default::default())).await?;

    assert_eq!(Ok(()), cnt.send(Null {}).await);

    cnt.link_mut()[0].break_down();
    assert_eq!(
        Err(AUTDDriverError::ConfirmResponseFailed),
        cnt.send(Null {}).await
    );

    cnt.link_mut()[0].repair();
    assert_eq!(Ok(()), cnt.send(Null {}).await);

    cnt.link_mut()[0].break_down();
    assert_eq!(
        Err(AUTDDriverError::ConfirmResponseFailed),
        cnt.send(Null {}).await
    );
    cnt[0].enable = false;
    assert_eq!(Ok(()), cnt.send(Null {}).await);

    Ok(())
}
