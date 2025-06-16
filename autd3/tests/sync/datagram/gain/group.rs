use std::collections::HashMap;

use autd3::{
    link::{Audit, AuditOption},
    prelude::*,
};

#[test]
fn only_for_enabled() -> anyhow::Result<()> {
    let mut autd = Controller::<_, firmware::Latest>::open_with(
        [AUTD3::default(), AUTD3::default()],
        Audit::latest(AuditOption::default()),
    )?;

    let check = std::sync::Arc::new(std::sync::Mutex::new(vec![false; autd.num_devices()]));

    autd.send(autd3_driver::datagram::Group::new(
        |dev| (dev.idx() == 1).then_some(()),
        HashMap::from([(
            (),
            gain::Group {
                key_map: |dev| {
                    check.lock().unwrap()[dev.idx()] = true;
                    move |_| Some(0)
                },
                gain_map: HashMap::from([(
                    0,
                    Uniform {
                        phase: Phase(0x90),
                        intensity: EmitIntensity(0x80),
                    },
                )]),
            },
        )]),
    ))?;

    assert!(!check.lock().unwrap()[0]);
    assert!(check.lock().unwrap()[1]);

    assert!(
        autd.link()[0]
            .fpga()
            .drives_at(Segment::S0, 0)
            .into_iter()
            .all(|d| Drive::NULL == d)
    );
    assert!(
        autd.link()[1]
            .fpga()
            .drives_at(Segment::S0, 0)
            .into_iter()
            .all(|d| Drive {
                phase: Phase(0x90),
                intensity: EmitIntensity(0x80)
            } == d)
    );

    Ok(())
}
