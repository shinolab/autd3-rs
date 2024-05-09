use autd3::prelude::*;

pub async fn custom(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::disable()).await?;

    let m = autd3::modulation::Custom::new(SamplingConfig::DISABLE, |dev| match dev.idx() {
        0 => Ok(vec![0xFF, 0xFF]),
        _ => Ok(vec![0x00, 0x00]),
    });
    let g = autd3::gain::Custom::new(|dev| {
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
