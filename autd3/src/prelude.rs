#[cfg(feature = "gain")]
pub use crate::gain::{
    self, Bessel, BesselOption, Focus, FocusOption, GainGroup, Null, Plane, PlaneOption, Uniform,
};

#[cfg(feature = "modulation")]
pub use crate::modulation::{
    self, Fir, FourierOption, RadiationPressure, Sine, SineOption, Square, SquareOption, Static,
};

#[cfg(feature = "stm")]
pub use crate::stm::{Circle, Line};

#[cfg(feature = "link-nop")]
pub use crate::link::Nop;

pub use crate::controller::{Controller, ParallelMode, SenderOption};

pub use autd3_core::{
    devices::AUTD3,
    firmware::{
        Drive, GPIOIn, GPIOOut, Intensity, Phase, PulseWidth, SamplingConfig, Segment,
        transition_mode,
    },
    modulation::Modulation,
};

pub use autd3_driver::{
    common::{Hz, PI, ULTRASOUND_FREQ, deg, kHz, mm, rad},
    datagram::{
        BoxedGain, Clear, ControlPoint, ControlPoints, FixedCompletionTime, FixedUpdateRate,
        FociSTM, ForceFan, GPIOOutputType, GPIOOutputs, GainSTM, GainSTMMode, GainSTMOption, Group,
        OutputMask, PhaseCorrection, PulseWidthEncoder, ReadsFPGAState, Silencer,
        SwapSegmentFociSTM, SwapSegmentGain, SwapSegmentGainSTM, SwapSegmentModulation,
        WithFiniteLoop, WithSegment,
    },
    error::AUTDDriverError,
    ethercat::DcSysTime,
    firmware::operation::BoxedDatagram,
    geometry::{EulerAngle, Geometry, Point3, Quaternion, UnitQuaternion, UnitVector3, Vector3},
};
