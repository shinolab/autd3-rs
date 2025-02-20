use std::time::Duration;

use autd3::{
    r#async::{AsyncSleeper, Controller},
    link::{Audit, AuditOption},
    prelude::*,
};
use autd3_core::link::LinkError;
use autd3_driver::firmware::{cpu::RxMessage, fpga::FPGAState};

#[tokio::test]
async fn audit_test() -> anyhow::Result<()> {
    let mut autd = Controller::open_with_option(
        [AUTD3::default()],
        Audit::new(AuditOption::default()),
        SenderOption::<AsyncSleeper> {
            timeout: Some(Duration::from_millis(10)),
            ..Default::default()
        },
    )
    .await?;
    assert_eq!(0, autd.link()[0].idx());

    {
        autd.sender(SenderOption::<AsyncSleeper> {
            timeout: Some(Duration::from_millis(20)),
            ..Default::default()
        })
        .send(Null {})
        .await?;
    }

    {
        assert_eq!(vec![None], autd.fpga_state().await?);
        assert!(autd.send(ReadsFPGAState::new(|_| true)).await.is_ok());
        autd.link_mut()[0].update();
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(0x88, 0x00))],
            autd.fpga_state().await?
        );
        autd.link_mut()[0].fpga_mut().assert_thermal_sensor();
        autd.link_mut()[0].update();
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(0x89, 0x00))],
            autd.fpga_state().await?
        );
    }

    {
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("broken"))),
            autd.send(Static::default()).await
        );
        assert_eq!(
            Err(AUTDError::Driver(AUTDDriverError::Link(LinkError::new(
                "broken"
            )))),
            autd.fpga_state().await
        );
        autd.link_mut().repair();
        assert!(autd.send(Static::default()).await.is_ok());
    }

    {
        use autd3_core::link::AsyncLink;
        assert!(autd.link_mut().close().await.is_ok());
        assert_eq!(
            Err(AUTDDriverError::LinkClosed),
            autd.send(Static::default()).await
        );
        assert_eq!(
            Err(AUTDError::Driver(AUTDDriverError::LinkClosed)),
            autd.fpga_state().await
        );
    }

    Ok(())
}
