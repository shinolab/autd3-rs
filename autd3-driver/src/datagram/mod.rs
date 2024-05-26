mod clear;
mod clk;
mod debug;
mod force_fan;
mod gain;
mod gpio_in;
mod modulation;
mod phase_filter;
mod pulse_width_encoder;
mod reads_fpga_state;
pub mod segment;
mod silencer;
// mod stm;
mod synchronize;
mod with_segment;
mod with_segment_transition;
mod with_timeout;

pub use clear::Clear;
pub use clk::ConfigureFPGAClock;
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{
    Gain, GainCache, GainCalcFn, GainFilter, GainTransform, Group, IntoGainCache, IntoGainTransform,
};
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    IntoModulationCache, IntoModulationTransform, IntoRadiationPressure, Modulation,
    ModulationCache, ModulationProperty, ModulationTransform, RadiationPressure,
};
pub use phase_filter::PhaseFilter;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use segment::SwapSegment;
pub use silencer::{Silencer, SilencerFixedCompletionSteps, SilencerFixedUpdateRate};
// pub use stm::{ControlPoint, FocusSTM, GainSTM};
pub use synchronize::Synchronize;
pub use with_segment::{DatagramS, DatagramWithSegment, IntoDatagramWithSegment};
pub use with_segment_transition::{
    DatagramST, DatagramWithSegmentTransition, IntoDatagramWithSegmentTransition,
};
pub use with_timeout::{DatagramWithTimeout, IntoDatagramWithTimeout};

use std::time::Duration;

use crate::{
    derive::{Device, Geometry},
    error::AUTDInternalError,
    firmware::operation::Operation,
};

pub trait Datagram<'a> {
    type O1: Operation + 'a;
    type O2: Operation + 'a;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

impl<'a, D1, D2> Datagram<'a> for (D1, D2)
where
    D1: Datagram<'a, O2 = crate::firmware::operation::NullOp>,
    D2: Datagram<'a, O2 = crate::firmware::operation::NullOp>,
{
    type O1 = D1::O1;
    type O2 = D2::O1;

    fn operation(
        &'a self,
        geometry: &'a Geometry,
    ) -> Result<impl Fn(&'a Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
        let o1 = self.0.operation(geometry)?;
        let o2 = self.1.operation(geometry)?;
        Ok(move |dev| (o1(dev).0, o2(dev).0))
    }

    fn timeout(&self) -> Option<Duration> {
        match (self.0.timeout(), self.1.timeout()) {
            (Some(t1), Some(t2)) => Some(t1.max(t2)),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            (None, None) => None,
        }
    }
}
