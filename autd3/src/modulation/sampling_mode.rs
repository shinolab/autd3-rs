use autd3_driver::{derive::EmitIntensity, error::AUTDInternalError};

pub trait SamplingMode: Clone {
    type D;
    fn calc(freq: f64, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
}
