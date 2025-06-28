use autd3::{
    Controller,
    firmware::V12_1,
    gain::Null,
    link::{Audit, AuditOption, audit::version},
    prelude::{AUTD3, AUTDDriverError},
};
use autd3_core::link::MsgId;

mod datagram;
mod link;

#[test]
fn initial_msg_id() -> anyhow::Result<()> {
    let cnt = Controller::<_, V12_1>::open_with(
        [AUTD3::default()],
        Audit::<version::V12_1>::new(AuditOption {
            initial_msg_id: Some(MsgId::new(0x01)),
            initial_phase_corr: Some(0xFF),
            ..Default::default()
        }),
    )?;

    assert!(
        cnt.link()[0]
            .fpga()
            .phase_correction()
            .iter()
            .all(|v| v.0 == 0x00)
    );

    Ok(())
}

#[test]
fn test_retry_with_disabled_device() -> anyhow::Result<()> {
    let mut cnt = Controller::<_, V12_1>::open_with(
        [AUTD3::default(); 2],
        Audit::<version::V12_1>::new(Default::default()),
    )?;

    assert_eq!(Ok(()), cnt.send(Null {}));

    cnt.link_mut()[0].break_down();
    assert_eq!(
        Err(AUTDDriverError::ConfirmResponseFailed),
        cnt.send(Null {})
    );

    cnt.link_mut()[0].repair();
    assert_eq!(Ok(()), cnt.send(Null {}));

    cnt.link_mut()[0].break_down();
    assert_eq!(
        Err(AUTDDriverError::ConfirmResponseFailed),
        cnt.send(Null {})
    );
    assert_eq!(
        Ok(()),
        cnt.send(autd3_driver::datagram::Group::new(
            |dev| (dev.idx() != 0).then_some(()),
            std::collections::HashMap::from([((), Null {})])
        ))
    );

    Ok(())
}
