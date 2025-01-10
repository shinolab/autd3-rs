use autd3::{
    link::Audit,
    prelude::{Point3, AUTD3},
    Controller,
};

mod datagram;
mod link;

#[test]
fn initial_msg_id() -> anyhow::Result<()> {
    let cnt = Controller::builder([AUTD3::new(Point3::origin())]).open(
        Audit::builder()
            .with_initial_msg_id(Some(0x01))
            .with_initial_phase_corr(Some(0xFF)),
    )?;

    assert!(cnt.link()[0]
        .fpga()
        .phase_correction()
        .iter()
        .all(|v| v.value() == 0x00));

    Ok(())
}
