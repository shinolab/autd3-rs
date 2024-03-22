mod focus;
mod gain;
mod props;
mod sampling_config;
mod segment;

pub use focus::FocusSTM;
pub use gain::GainSTM;
pub use props::STMProps;
pub use segment::{ChangeFocusSTMSegment, ChangeGainSTMSegment};
