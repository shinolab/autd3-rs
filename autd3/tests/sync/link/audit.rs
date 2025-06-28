use std::time::Duration;

use autd3::{
    controller::{FixedSchedule, SenderOption},
    link::{Audit, AuditOption, audit::version},
    prelude::*,
};
use autd3_core::link::{Ack, LinkError, RxMessage};
use autd3_driver::firmware::v12_1::fpga::FPGAState;

#[test]
fn audit_test() -> anyhow::Result<()> {
    let mut autd = Controller::<_, firmware::V12_1>::open_with_option(
        [AUTD3::default()],
        Audit::<version::V12_1>::new(AuditOption::default()),
        SenderOption {
            timeout: Some(Duration::from_millis(10)),
            ..Default::default()
        },
        FixedSchedule::default(),
    )?;
    assert_eq!(0, autd.link()[0].idx());

    {
        autd.sender(
            SenderOption {
                timeout: Some(Duration::from_millis(20)),
                ..Default::default()
            },
            FixedSchedule::default(),
        )
        .send(Null {})?;
    }

    {
        assert_eq!(vec![None], autd.fpga_state()?);
        assert!(autd.send(ReadsFPGAState::new(|_| true)).is_ok());
        autd.link_mut()[0].update();
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(0x88, Ack::new()))],
            autd.fpga_state()?
        );
        autd.link_mut()[0].fpga_mut().assert_thermal_sensor();
        autd.link_mut()[0].update();
        assert_eq!(
            vec![FPGAState::from_rx(&RxMessage::new(0x89, Ack::new()))],
            autd.fpga_state()?
        );
    }

    {
        autd.link_mut().break_down();
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("broken"))),
            autd.send(Static::default())
        );
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::new("broken"))),
            autd.fpga_state()
        );
        autd.link_mut().repair();
        assert!(autd.send(Static::default()).is_ok());
    }

    {
        use autd3_core::link::Link;
        assert!(autd.link_mut().close().is_ok());
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            autd.send(Static::default())
        );
        assert_eq!(
            Err(AUTDDriverError::Link(LinkError::closed())),
            autd.fpga_state()
        );
    }

    Ok(())
}
