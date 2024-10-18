use std::time::Duration;

use autd3::{derive::DEFAULT_TIMEOUT, link::Audit, prelude::*};
use autd3_driver::firmware::{cpu::RxMessage, fpga::FPGAState};

#[tokio::test]
async fn audit_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .with_fallback_timeout(Duration::from_millis(100))
        .open(Audit::builder())
        .await?;
    assert_eq!(Duration::from_millis(100), autd.fallback_timeout());
    assert_eq!(0, autd.link()[0].idx());
    assert_eq!(DEFAULT_TIMEOUT, autd.link().last_timeout());
    assert_eq!(usize::MAX, autd.link().last_parallel_threshold());

    // test last_timeout and last_parallel_threshold
    {
        assert!(autd.send(Null::new()).await.is_ok());
        assert_eq!(Duration::from_millis(100), autd.link().last_timeout());
        assert_eq!(4, autd.link().last_parallel_threshold());

        assert!(autd.send(Static::new()).await.is_ok());
        assert_eq!(DEFAULT_TIMEOUT, autd.link().last_timeout());
        assert_eq!(usize::MAX, autd.link().last_parallel_threshold());
    }

    // test fpga_state
    {
        assert_eq!(vec![None], autd.fpga_state().await?);
        assert!(autd.send(ReadsFPGAState::new(|_| true)).await.is_ok());
        autd.link_mut()[0].update();
        assert_eq!(
            vec![Option::<FPGAState>::from(&RxMessage::new(0x88, 0x00))],
            autd.fpga_state().await?
        );
        autd.link_mut()[0].fpga_mut().assert_thermal_sensor();
        autd.link_mut()[0].update();
        assert_eq!(
            vec![Option::<FPGAState>::from(&RxMessage::new(0x89, 0x00))],
            autd.fpga_state().await?
        );
    }

    {
        autd.link_mut().down();
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::SendDataFailed)),
            autd.send(Static::new()).await
        );
        assert_eq!(Err(AUTDError::ReadFPGAStateFailed), autd.fpga_state().await);
        autd.link_mut().up();
        assert!(autd.send(Static::new()).await.is_ok());
        autd.link_mut().break_down();
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
        autd.link_mut().repair();
        assert!(autd.send(Static::new()).await.is_ok());
    }

    {
        use autd3_driver::link::Link;
        assert!(autd.link_mut().close().await.is_ok());
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::LinkClosed)),
            autd.send(Static::new()).await
        );
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::LinkClosed)),
            autd.fpga_state().await
        );
    }

    Ok(())
}
