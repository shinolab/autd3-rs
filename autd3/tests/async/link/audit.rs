use std::time::Duration;

use autd3::{
    r#async::{AsyncSleeper, Controller},
    controller::SenderOption,
    link::{Audit, AuditOption},
    prelude::*,
};
use autd3_core::link::{Ack, LinkError, RxMessage};
use autd3_driver::firmware::fpga::FPGAState;

#[tokio::test]
async fn audit_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut autd = Controller::open_with(
        [AUTD3::default()],
        Audit::new(AuditOption::default()),
        SenderOption {
            timeout: Some(Duration::from_millis(10)),
            ..Default::default()
        },
        AsyncSleeper,
    )
    .await?;
    assert_eq!(0, autd.link()[0].idx());

    {
        autd.sender(
            SenderOption {
                timeout: Some(Duration::from_millis(20)),
                ..Default::default()
            },
            AsyncSleeper,
        )
        .send(Null {})
        .await?;
    }

    {
        assert_eq!(vec![None], autd.fpga_state().await?);
        assert!(autd.send(ReadsFPGAState::new(|_| true)).await.is_ok());
        autd.link_mut()[0].update_with_sys_time(DcSysTime::ZERO);
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(
                0x88,
                Ack::new(0x00, 0x00)
            ))],
            autd.fpga_state().await?
        );
        autd.link_mut()[0].fpga_mut().assert_thermal_sensor();
        autd.link_mut()[0].update_with_sys_time(DcSysTime::ZERO);
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(
                0x89,
                Ack::new(0x00, 0x00)
            ))],
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
            Err(AUTDDriverError::Link(LinkError::new("broken"))),
            autd.fpga_state().await
        );
        autd.link_mut().repair();
        assert!(autd.send(Static::default()).await.is_ok());
    }

    {
        use autd3_core::link::AsyncLink;
        assert!(autd.link_mut().close().await.is_ok());
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            autd.send(Static::default()).await
        );
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            autd.fpga_state().await
        );
    }

    Ok(())
}
