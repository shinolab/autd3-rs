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
mod stm;
mod synchronize;
mod with_segment;
mod with_segment_transition;
mod with_timeout;

pub use clear::Clear;
pub use clk::ConfigureFPGAClock;
pub use debug::DebugSettings;
pub use force_fan::ForceFan;
pub use gain::{
    Gain, GainCache, GainCalcResult, GainOperationGenerator, GainTransform, Group, IntoGainCache,
    IntoGainTransform,
};
pub use gpio_in::EmulateGPIOIn;
pub use modulation::{
    IntoModulationCache, IntoModulationTransform, IntoRadiationPressure, Modulation,
    ModulationCache, ModulationCalcResult, ModulationOperationGenerator, ModulationProperty,
    ModulationTransform, RadiationPressure,
};
pub use phase_filter::PhaseFilter;
pub use pulse_width_encoder::PulseWidthEncoder;
pub use reads_fpga_state::ReadsFPGAState;
pub use segment::SwapSegment;
pub use silencer::{Silencer, SilencerFixedCompletionSteps, SilencerFixedUpdateRate};
pub use stm::{FocusSTM, GainSTM};
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

pub trait OperationGenerator<'a>: Send + Sync {
    type O1: Operation + 'a;
    type O2: Operation + 'a;
    fn generate(&'a self, device: &'a Device) -> Result<(Self::O1, Self::O2), AUTDInternalError>;
}

pub trait Datagram<'a> {
    type O1: Operation + 'a;
    type O2: Operation + 'a;
    type G: OperationGenerator<'a, O1 = Self::O1, O2 = Self::O2>;

    fn operation_generator(self, geometry: &'a Geometry) -> Result<Self::G, AUTDInternalError>;

    fn timeout(&self) -> Option<Duration> {
        None
    }
}

pub struct CombinedOperationGenerator<'a, O1, O2>
where
    O1: OperationGenerator<'a>,
    O2: OperationGenerator<'a>,
{
    o1: O1,
    o2: O2,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, O1, O2> OperationGenerator<'a> for CombinedOperationGenerator<'a, O1, O2>
where
    O1: OperationGenerator<'a>,
    O2: OperationGenerator<'a>,
{
    type O1 = O1::O1;
    type O2 = O2::O1;

    fn generate(
        &'a self,
        device: &'a Device,
    ) -> Result<
        (
            <O1 as OperationGenerator>::O1,
            <O2 as OperationGenerator>::O1,
        ),
        AUTDInternalError,
    > {
        let (o1, _) = self.o1.generate(device)?;
        let (o2, _) = self.o2.generate(device)?;
        Ok((o1, o2))
    }
}

impl<'a, D1, D2> Datagram<'a> for (D1, D2)
where
    D1: Datagram<'a, O2 = crate::firmware::operation::NullOp>,
    D2: Datagram<'a, O2 = crate::firmware::operation::NullOp>,
{
    type O1 = D1::O1;
    type O2 = D2::O1;
    type G = CombinedOperationGenerator<'a, D1::G, D2::G>;

    fn operation_generator(self, geometry: &'a Geometry) -> Result<Self::G, AUTDInternalError> {
        Ok(CombinedOperationGenerator {
            o1: self.0.operation_generator(geometry)?,
            o2: self.1.operation_generator(geometry)?,
            _phantom: std::marker::PhantomData,
        })
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
