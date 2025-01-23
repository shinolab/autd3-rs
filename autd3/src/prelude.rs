pub use crate::{
    controller::{Controller, ParallelMode, SenderOption},
    datagram::{
        gain::{
            Bessel, BesselOption, Focus, FocusOption, Group, Null, Plane, PlaneOption, Uniform,
        },
        modulation::{FourierOption, Sine, SineOption, Square, SquareOption, Static},
        stm::{Circle, Line},
    },
    error::AUTDError,
    link::Nop,
};

pub use autd3_core::modulation::Modulation;

pub use autd3_driver::{
    autd3_device::AUTD3,
    datagram::{
        Clear, ControlPoint, ControlPoints, DebugSettings, FixedUpdateRate, FociSTM, ForceFan,
        GainSTM, GainSTMOption, IntoBoxedGain, IntoBoxedModulation, PhaseCorrection,
        PulseWidthEncoder, ReadsFPGAState, Silencer, SwapSegment,
    },
    defined::{deg, kHz, mm, rad, ultrasound_freq, Hz, PI},
    error::AUTDDriverError,
    ethercat::DcSysTime,
    firmware::{
        cpu::GainSTMMode,
        fpga::{
            DebugType, Drive, EmitIntensity, GPIOIn, GPIOOut, LoopBehavior, Phase, SamplingConfig,
            Segment, SilencerTarget, TransitionMode,
        },
    },
    geometry::{EulerAngle, Geometry, Point3, Quaternion, UnitQuaternion, UnitVector3, Vector3},
};

#[cfg(not(feature = "dynamic_freq"))]
pub use autd3_driver::datagram::FixedCompletionTime;
