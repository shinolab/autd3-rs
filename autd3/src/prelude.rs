#[cfg(feature = "gain")]
pub use crate::datagram::gain::{
    self, Bessel, BesselOption, Focus, FocusOption, GainGroup, Null, Plane, PlaneOption, Uniform,
};

#[cfg(feature = "modulation")]
pub use crate::datagram::modulation::{
    self, Fir, FourierOption, RadiationPressure, Sine, SineOption, Square, SquareOption, Static,
};

#[cfg(feature = "stm")]
pub use crate::datagram::stm::{Circle, Line};

#[cfg(feature = "link-nop")]
pub use crate::link::Nop;

pub use crate::controller::{Controller, ParallelMode, SenderOption};

pub use autd3_core::{
    firmware::{
        Drive, GPIOIn, GPIOOut, Intensity, Phase, PulseWidth, SamplingConfig, Segment,
        transition_mode,
    },
    modulation::Modulation,
    sleep::{self, StdSleeper},
};

pub use autd3_driver::{
    autd3_device::AUTD3,
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
