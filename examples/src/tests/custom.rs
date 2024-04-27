use autd3::prelude::*;

pub async fn custom(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::disable()).await?;

    let m = Static::new();
    let g = Custom::new(|dev| {
        let dev_idx = dev.idx();
        move |tr| match (dev_idx, tr.idx()) {
            (0, 0) => Drive::new(Phase::new(0), EmitIntensity::new(0xFF)),
            (0, 248) => Drive::new(Phase::new(0), EmitIntensity::new(0xFF)),
            _ => Drive::null(),
        }
    });

    autd.send((m, g)).await?;

    Ok(true)
}
