use autd3_driver::defined::ULTRASOUND_PERIOD;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalcError {
    #[error("Recording is already started")]
    RecordingAlreadyStarted,
    #[error("Recording is not started")]
    RecodingNotStarted,
    #[error("Tick must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidTick,
}
