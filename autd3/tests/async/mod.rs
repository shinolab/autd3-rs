use autd3::{
    link::Audit,
    prelude::{Point3, AUTD3},
    r#async::Controller,
};

mod datagram;
mod link;

#[tokio::test]
async fn initial_msg_id() -> anyhow::Result<()> {
    let cnt = Controller::builder([AUTD3::default()])
        .open(
            Audit::builder(AuditOption::default())
                .with_initial_msg_id(Some(0x01))
                .with_initial_phase_corr(Some(0xFF)),
        )
        .await?;

    assert!(cnt.link()[0]
        .fpga()
        .phase_correction()
        .iter()
        .all(|v| v.0 == 0x00));

    Ok(())
}
