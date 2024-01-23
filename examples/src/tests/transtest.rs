use autd3::prelude::*;

pub async fn transtest<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::disable()).await?;

    let m = Static::new();
    let g = TransducerTest::new(|dev, tr| match (dev.idx(), tr.idx()) {
        (0, 0) => Some(Drive {
            phase: Phase::new(0),
            intensity: EmitIntensity::new(0xFF),
        }),
        (0, 248) => Some(Drive {
            phase: Phase::new(0),
            intensity: EmitIntensity::new(0xFF),
        }),
        _ => None,
    });

    autd.send((m, g)).await?;

    Ok(true)
}
