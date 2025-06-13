use autd3::{
    Controller,
    gain::Null,
    link::{Audit, AuditOption, audit::version},
    prelude::{AUTD3, AUTDDriverError},
};

#[test]
fn firmware_v11() -> anyhow::Result<()> {
    let _cnt = Controller::open(
        [AUTD3::default()],
        Audit::<version::V11>::new(AuditOption::default()),
    )?;

    Ok(())
}
