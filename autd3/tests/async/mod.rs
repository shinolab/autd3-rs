use autd3::{
    r#async::Controller,
    link::{Audit, AuditOption},
    prelude::AUTD3,
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
