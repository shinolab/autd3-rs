use autd3::{core::link::Link, prelude::*};

pub fn audio_file(autd: &mut Controller<impl Link, firmware::Auto>) -> anyhow::Result<bool> {
    autd.send(Silencer::default())?;

    let center = autd.center() + Vector3::new(0., 0., 150.0 * mm);

    let g = Focus {
        pos: center,
        option: Default::default(),
    };
    const WAV_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/resources/sin150.wav");
    let m = autd3_modulation_audio_file::Wav::new(WAV_FILE)?;

    autd.send((m, g))?;

    Ok(true)
}
