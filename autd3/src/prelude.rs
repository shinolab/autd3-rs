/*
 * File: prelude.rs
 * Project: src
 * Created Date: 27/04/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 08/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

pub use crate::gain::IntoCache as IntoGainCache;
pub use crate::gain::IntoTransform as IntoGainTransform;
pub use crate::modulation::IntoCache as IntoModulationCache;
pub use crate::modulation::IntoTransform as IntoModulationTransform;

pub use crate::{
    controller::Controller,
    error::AUTDError,
    gain::{Bessel, Focus, Group, Null, Plane, TransducerTest, Uniform},
    link::Nop,
    modulation::{IntoRadiationPressure, SamplingMode, Sine, Square, Static},
};

pub use autd3_driver::{
    autd3_device::AUTD3,
    common::Rad as PhaseRad,
    common::{Drive, EmitIntensity, Phase, SamplingConfiguration},
    datagram::{
        Clear, ConfigureDebugOutputIdx, ConfigureForceFan, ConfigureModDelay,
        ConfigureReadsFPGAInfo, DatagramT, FocusSTM, GainSTM, Modulation, ModulationProperty,
        Silencer, Stop, Synchronize,
    },
    defined::{float, METER, MILLIMETER, PI},
    error::AUTDInternalError,
    fpga::FPGA_CLK_FREQ,
    geometry::*,
    link::Link,
    operation::{ControlPoint, GainSTMMode},
    timer_strategy::TimerStrategy,
};
