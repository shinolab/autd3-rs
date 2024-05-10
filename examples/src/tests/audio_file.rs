use autd3::prelude::*;

pub async fn audio_file(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(Silencer::default()).await?;

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus::new(center);
    const WAV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/sin150.wav");
    let m = autd3_modulation_audio_file::Wav::new(WAV_FILE);

    // const WAV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/sin150.dat");
    // let m = autd3_modulation_audio_file::RawPCM::new(WAV_FILE, 4000);

    autd.send((m, g)).await?;

    Ok(true)
}
