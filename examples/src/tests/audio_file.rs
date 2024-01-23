use autd3::prelude::*;

pub async fn audio_file<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<bool> {
    autd.send(ConfigureSilencer::default()).await?;

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);

    let g = Focus::new(center);
    const WAV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/sin150.wav");
    let m = autd3_modulation_audio_file::Wav::new(WAV_FILE)
        .map_err(|e| AUTDError::Internal(e.into()))?;

    // const WAV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/sin150.dat");
    // let m = autd3_modulation_audio_file::RawPCM::new(WAV_FILE, 4000)
    //     .map_err(|e| AUTDError::Internal(e.into()))?;

    autd.send((m, g)).await?;

    Ok(true)
}
