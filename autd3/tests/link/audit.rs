use autd3::{derive::DEFAULT_TIMEOUT, link::Audit, prelude::*};
use autd3_driver::{
    firmware::{cpu::RxMessage, fpga::FPGAState},
    link::Link,
};

#[tokio::test]
async fn audit_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .open(Audit::builder().with_timeout(std::time::Duration::from_millis(100)))
        .await?;
    assert_eq!(std::time::Duration::from_millis(100), autd.link.timeout());

    assert_eq!(0, autd.link.emulators()[0].idx());
    assert_eq!(0, autd.link[0].idx());
    assert_eq!(Some(DEFAULT_TIMEOUT), autd.link.last_timeout());

    assert_eq!(vec![None], autd.fpga_state().await?);
    autd.send(ReadsFPGAState::new(|_| true)).await?;
    autd.link[0].update();
    assert_eq!(
        vec![Option::<FPGAState>::from(&RxMessage::new(0x00, 0x88))],
        autd.fpga_state().await?
    );
    autd.link.emulators_mut()[0]
        .fpga_mut()
        .assert_thermal_sensor();
    autd.link[0].update();
    assert_eq!(
        vec![Option::<FPGAState>::from(&RxMessage::new(0x00, 0x89))],
        autd.fpga_state().await?
    );

    autd.link.down();
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::SendDataFailed)),
        autd.send(Static::new()).await
    );
    assert_eq!(Err(AUTDError::ReadFPGAStateFailed), autd.fpga_state().await);
    autd.link.up();
    autd.send(Static::new()).await?;
    autd.link.break_down();
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::LinkError(
            "broken".to_string()
        ))),
        autd.send(Static::new()).await
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::LinkError(
            "broken".to_string()
        ))),
        autd.fpga_state().await
    );
    autd.link.repair();
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
