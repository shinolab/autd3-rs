use autd3_driver::{derive::EmitIntensity, error::AUTDInternalError};

pub trait SamplingMode: Clone {
    type F: Copy;
    type D;
    fn calc(freq: Self::F, data: Self::D) -> Result<Vec<EmitIntensity>, AUTDInternalError>;
}
