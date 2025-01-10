use std::time::Duration;

use autd3::{link::Audit, prelude::*};
use autd3_driver::firmware::{cpu::RxMessage, fpga::FPGAState};

#[test]
fn audit_test() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Point3::origin())])
        .with_default_timeout(Duration::from_millis(100))
        .open_with_timeout(Audit::builder(), Duration::from_millis(10))?;
    assert_eq!(Some(Duration::from_millis(10)), autd.link().last_timeout());
    assert_eq!(Some(usize::MAX), autd.link().last_parallel_threshold());
    assert_eq!(Duration::from_millis(100), autd.timer().default_timeout());
    assert_eq!(0, autd.link()[0].idx());

    {
        autd.send(
            Null::new()
                .with_parallel_threshold(Some(1))
                .with_timeout(Some(Duration::from_millis(20))),
        )?;
        assert_eq!(Some(Duration::from_millis(20)), autd.link().last_timeout());
        assert_eq!(Some(1), autd.link().last_parallel_threshold());

        autd.send(Null::new().with_parallel_threshold(None).with_timeout(None))?;
        assert_eq!(None, autd.link().last_timeout());
        assert_eq!(None, autd.link().last_parallel_threshold());
    }

    {
        assert_eq!(vec![None], autd.fpga_state()?);
        assert!(autd.send(ReadsFPGAState::new(|_| true)).is_ok());
        autd.link_mut()[0].update();
        assert_eq!(
            vec![Option::<FPGAState>::from(&RxMessage::new(0x88, 0x00))],
            autd.fpga_state()?
        );
        autd.link_mut()[0].fpga_mut().assert_thermal_sensor();
        autd.link_mut()[0].update();
        assert_eq!(
            vec![Option::<FPGAState>::from(&RxMessage::new(0x89, 0x00))],
            autd.fpga_state()?
        );
    }

    {
        autd.link_mut().down();
        assert_eq!(
            Err(AUTDDriverError::SendDataFailed),
            autd.send(Static::new())
        );
        assert_eq!(Err(AUTDError::ReadFPGAStateFailed), autd.fpga_state());
        autd.link_mut().up();
        assert!(autd.send(Static::new()).is_ok());
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDDriverError::LinkError("broken".to_string())),
            autd.send(Static::new())
        );
        assert_eq!(
            Err(AUTDError::Driver(AUTDDriverError::LinkError(
                "broken".to_string()
            ))),
            autd.fpga_state()
        );
        autd.link_mut().repair();
        assert!(autd.send(Static::new()).is_ok());
    }

    {
        use autd3_driver::link::Link;
        assert!(autd.link_mut().close().is_ok());
        assert_eq!(Err(AUTDDriverError::LinkClosed), autd.send(Static::new()));
        assert_eq!(
            Err(AUTDError::Driver(AUTDDriverError::LinkClosed)),
            autd.fpga_state()
        );
    }

    Ok(())
}
