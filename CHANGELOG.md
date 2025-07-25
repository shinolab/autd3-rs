# Changelog


# 35.0.1 (2025-06-30)

## 🐛 Bug Fixes

- Sending a gain causes `ConfirmResponseFailed` error with `Simulator` link (#310)


# 35.0.0 (2025-06-30)

## 🚀 Features

- Add `OutputMask` datagram
- [**breaking**] Support firmware v12.1
- [**breaking**] Transfer `sound_speed` from `Device` to `Environment` (#305)
- [**breaking**] Make `link::Audit` and `link::Nop` optional

## 🚜 Refactor

- [**breaking**] Rename from `FixedCompletionTime::strict_mode` to `FixedCompletionTime::strict`
- [**breaking**] Change some `Datagram`s trait bound to remove boilerplates

## ⚡ Performance

- [**breaking**] Pass buffer to avoid allocation in firmware-emulator


# 34.0.0 (2025-06-23)

## 🚀 Features

- Add `left_handed` and `use_meter` features to `autd3` crate


# 34.0.0-rc.0 (2025-06-19)

## 🚀 Features

- Add `thread-safe` feature to make `CPUEmulator` `Sync` (#272)
- Introduce `GainGroup` alias for gain::Group`
- Add `SpinWaitSleeper`
- [**breaking**] Add `TimerStrategy` (#282)
- Support firmware v12.0.0 (#289)
- Add supports for old firmwares

## 🗑️ Removals

- [**breaking**] `gain::Cache` and `modulation::Cache`
- [**breaking**] `Device::enable` flag (#275)
- [**breaking**] `IntoBoxed` traits
- [**breaking**] `WaitableSleeper`
- [**breaking**] `lightweight` feature

## 🐛 Bug Fixes

- `datagram::Group` unnecessarily updates `Geometry::version` (#263)
- `SenderOption::timeout` reverts to `None` (#264)
- Error when sending invalid msg id should be `InvalidMessageID` rather than  `ConfirmResponseFailed` (#290)

## 🚜 Refactor

- Move codes related to transmission control to `autd3-driver` (#286)
- [**breaking**] Rename from `EmitIntensity` to `Intensity`

## ⚡ Performance

- Use `ReadsFPGAState` instead of `ForceFan` to clear `msg_id` in device
- Adopt `smallvec`
- Adjust parallel threshold (#285)

## ⚙️ Miscellaneous Tasks

- Update error messages


# 33.0.0 (2025-05-25)

## 🚀 Features

- Make `SenderOption` used in `Controller::send` configurable (#250)
- Enhance `Greedy` algorithm with customizable objective function (#253)
- Make `GPIOOutputType` optional
- Add `BoxedGain::new`, `BoxedModulation::new` and `BoxedDatagram::new`
- Add `Controller::inspect` (#261)

## 🗑️ Removals

- `Controller::group_send`, add `datagram::Group` instead (#257)

## ⬆️ Update Dependencies

- Update bitfield-struct requirement from 0.10.1 to 0.11.0
- Update criterion requirement from 0.5.1 to 0.6.0

## 🐛 Bug Fixes

- No need to check the response of disabled devices (#247)

## 🚜 Refactor

- Rename from `division` to `divide` in sampling config
- Rename from `Greedy::phase_div` to `Greedy::phase_quantization_levels`
- Rename from `defined` module to `common`
- Stop using `tonic::Status` in `AUTDProtoBufError::Status` to avoid `clippy::result_large_err`

## ⚡ Performance

- Transfer management of tx buffer from `Controller` to `Link` (#251)

## ⚙️ Miscellaneous Tasks

- Modify error message
- Correct various typos


# 32.1.1 (2025-04-21)

## 🐛 Bug Fixes

- Bump dependencies version


# 32.1.0 (2025-04-16)

## 🚀 Features

- Supports `PulseWidthEncoder` for v10 firmware

## 🐛 Bug Fixes

- Size calculations in `PulseWidthEncoderOp` is invalid


# 32.0.1

- Remove unintended `dbg!` 

# 32.0.0

- Update firmware to v11.0.0
  - Optimize STM memory usage: supports 65536 foci in total, regardless of the number of foci per pattern
  - Modulation buffer size increased from 32768 to 65536
  - The period of ultrasound is changed from 256 to 512
      - The maximum value of pulse width in `PulseWidthEncoder` is changed from 255 to 511
  - Remove `SilencerTarget`
- Rename from `DebugType` to `GPIOOutputType`
- Prereserve `enable` and `sound_speed` in `Geometry::reconfigure`
- Impl `expected_radiation_pressure` for modulations

# 31.0.0

- Remove `Device::translate`, `Device::translate_to`, `Device::rotate`, `Device::rotate_to`, and `Device::affine`
  - Add `Geometry::reconfigure` instead
- Rename from `DebugSettings` to `GPIOOutputs`
- Extensive refactoring and modification of the weight mode, and addition of features

# 30.0.1

- Fix `PartialEq` implementation for `SamplingConfig`

# 30.0.0

- `Modulation::sampling_config` now returns `SamplingConfig` instead of `Result<SamplingConfig, ModulationError>`
- Remove `SamplingConfig::new_nearest`, add `SamplingConfig::into_nearest` instead
- Update Protocol Buffers definitions

# 29.0.0

- Update firmware to v10.0.1
  - phase correction bram and pulse width encoder table reset to default in clear op
  - support for `dynamic_freq` version
- Make `async` optional
- Make `Transducer::new` public
- Make `autd3_gain_holo::kPa` public
- Rename from `AUTDInternalError` to `AUTDDriverError`
- Use `Point3` instead of `Vector3` for coordinate values
- Use `Vec<u8>` instead of `Arc<Vec<u8>>` in `Modulation`s
- `SwapSegment::Gain` now take `TransitionMode` instead of `bool`
  - Still, `TransitionMode::Immediate` is only supported
- Impl `IntoIterator` for `&Controller`
- Improve calculation performance of `Gain`s
  - Change custom `Gain` APIs
- Improve performance of `Geometry::center`
- Update tracing messages
- Add `Sender`
  - Move `send_interval`, `receive_interval`, `timeout`, and `parallel_threshold` options to `Sender`
- Add `Circle` and `Line` utilities for `FociSTM` and `GainSTM`
- Add all euler angle variants to `EulerAngle`
- Add `AUTDDriverError::UnusedKey` errors
  - `Controller::group_send` and `gain::Group::init` now return `AUTDDriverError::UnusedKey` if the key is not used
- Remove `Controller::group`, use `Controller::group_send` instead
- Remove `LinkBuilder`, use `Link::new` instead
- Remove `with_xxx` methods to set option value, add option struct instead
- Remove resampler
- Remove `IntoCache`, `IntoFir`, and `IntoRadiationPressure` traits
- Remove `Deref<Target = Link>` and `DerefMut` for `Controller`
  - Impl `Deref<Target = Geometry>` and `DerefMut` for `Controller` instead
- Remove `Gain::with_transform`
- Remove `parallel` option from `gain::Group`
- Remove `Drive::null`, add `Drive::NULL` instead
- Remove `Silencer::is_valid`
- Remove `RawPCM` modulation
- Fix `Sine` and `Fourier`'s `offset`
- Fix `Controller::group` for `Gain`s which cannot be calculated independently for each device, such as `Gain`s in `autd3-gain-holo`
- Fix [#130](https://github.com/shinolab/autd3-rs/issues/130): `Gain`s in `autd3-gain-holo` cause `index out of bounds` error with disabled device
- Fix [#140](https://github.com/shinolab/autd3-rs/issues/140): Clear sometimes fails in `Controller::open`
- Fix [#197](https://github.com/shinolab/autd3-rs/issues/197): `Controller::group` causes access violation with `Naive` gain

# 28.1.0

- Add `EulerAngle::XYZ`
- Add `offset` parameter to `Fourier`
- Add `IntoFir` into `autd3::prelude`
- Re-export `autd3_link_soem::ThreadPriorityValue`

# 28.0.1

- Fix [#122](https://github.com/shinolab/autd3-rs/issues/122): `NalgebraBackend::new` is missing

# 28.0.0

- Update firmware to v10.0.0
  - Add `PhaseCorrection`
  - Update Silencer update rate from 8bit to 16bit
  - Add `DebugType::SysTimeEq`
- Remove `modulation::Mixer` and `modulation::Transform`
- Update some `Modulation`s API
  - Add `clamp` option to `Sine` and `Fourier`
  - Add `scale_factor` option to `Fourier`
  - `Sine::offset` value influence halved
- Change `Silencer` constructor
- `Controller::close` now take ownership of `self`
- Add `receive_interval` option to `Controller`
- Impl `Deref<Target = Link>` and `DerefMut` for `Controller`
- Change `Controller::timer_resolution` to optional
- Add `Transducer::dev_idx`
- Add `Phase::ZERO`
- Add `resampler`
  - Add `<Custom, Csv, RawPCM, Wave>::new_with_resampler`
  - Add `SincInterpolation` resampler and some window: `Blackman` and `Rectangular`
- Add `Fir` modulation
  - Add `with_fir` to `Modulation`s
- Fix bugs
  - Modulation with odd size causes unaligned memory access
  - Fix [#118](https://github.com/shinolab/autd3-rs/issues/118): Sometimes failed to open `Controller`

# 27.0.0

- Update firmware to v9.0.0
  - Add `SilencerTarget::Intensity` and `SilencerTarget::PulseWidth`
  - Change pulse width encoder table size to 256
  - Change ultrasound period to 256 from 512
  - Invert the sign of the phase
  - Remove `dynamic_freq` feature
- `Controller::link` and `geometry` is now private. use `link()`, `link_mut()`, `geometry()`, and `geometry_mut()` instead
- Unified `FociSTM` and `GainSTM` constructor to `new` and `new_nearest`
- Remove `Silencer::from_completion_steps`, use `Silencer::from_completion_time` instead
- Change the maximum value of `LoopBehavior` to 65535 from 4294967295
- Change base frequency of `SamplingConfig` to 40kHz from 20.48MHz
- Change to use `NonZero<T>` for integer argment which should be non-zero
- Change `Simulator::builder` argument from port number to `SocketAddr`
- Make `gain::Group` parallelism controllable
- Remove `autd3_gain_holo::SDP`
- `Directivity::directivity` now takes `Angle` as an argument instead of `f32`
- Add `Phase::PI`
- Implement `From<(Phase, EmitIntensity)>`, `From<(EmitIntensity, Phase)>`, `From<Phase>`, and `From<EmitIntensity>` for `Drive`
  - `impl Into<Drive>` can now be used instead of `Drive`
- Fix bugs
  - Remove unaligned reference

# 26.0.0

- Remove deprecated functions and structs
  - Remove `with_sampling_config` from `Wav`, `RawPCM`, `Csv`, and `Custom`
- Fix `DatagramWithSegmentTransition::trace` and `DatagramWithSegment::trace`
- Fix transition mode in lightweight mode
- Remove `EmissionConstraint::DontCare`
  - Use `EmissionConstraint::Clamp(EmitIntensity::MIN, EmitIntensity::MAX)` instead
- Rename `Silencer::fixed_xxx` to `Silencer::from_xxx`
- Add `Silencer::from_completion_time`
- Add `SamplingConfig::Period` and `SamplingConfig::PeriodNearest`
- Add `from_period` and `from_period_nearest` to `FociSTM` and `GainSTM`
- Add `freq` and `period` to `FociTM` and `GainSTM`
- Add `period` to `SamplingConfig`
- Functions that take `SamplingConfig` as an argument can now take `impl Into<SamplingConfig>` as an argument
  - Implement `Into<SamplingConfig>` for `Freq<u32>` and `Duration`
- Remove `ControllerBuilder::with_ultrasound_freq`
  - Set environment variable `AUTD3_ULTRASOUND_FREQ` instead with enabling `dynamic_freq` feature
- Update error messages and add logging for debugging
- Add `#[non_exhaustive]` attribute to enums
- Remove `&Geometry` argument from `Modulation::calc`

# 25.3.2

- Check synchronization before SafeOp. Also, activate Sync0 at the transition from PreOp to SafeOp.

# 25.3.1

- Fix `Controller::last_parallel_threshold()` 

# 25.3.0

- Add `last_parallel_threshold` to `Controller` for debugging
- `Custom::with_sampling_config` is now deprecated
- Improve `gain::Cache` performance
- Easing trait bound of `gain::Transform`
- Stop resampling in `Wav` modulation
- SOEM link now waits for system time to synchronize
  - Add `with_sync_tolerance` and `with_sync_timeout` for `SOEMBuilder`

# 25.2.3

- Fix `trace` for boxed Gains and Modulations 

# 25.2.2

- Fix `CPUEmulator::update`

# 25.2.1

- Fix `close` handling: `close` returns error if modulation with invalid sampling configuration was send

# 25.2.0

- Add `send_interval` to `ControllerBuilder`
- Add `timer_resolution` to `ControllerBuilder` on Windows
- Add tracing for debugging

# 25.1.0

- Update firmware to v8.0.1
  - Fix `FociSTM` op
- Add `repc(C)` for `ControlPoint` and `ControlPoints`
- `with_sampling_config` of `RawPCM` and `Csv` is now deprecated
- Trim records in `Csv`

# 25.0.1

- Fix `gain::Group`

# 25.0.0

- Update firmware to v8.0.0
  - Remove `PhaseFilter`
  - `FocusSTM` is now `FociSTM` with maximum 8 foci
  - `PulseWidthEncoder` table size is now shrinked to 32768
- Remove deprecated functions and structs
  - Remove `gain::Bessel2` and `with_transform2`
- Improve performance
  - Use `f32` intead of `f64`
  - All transmission data calculation steps can now be parallelized per device
    - Add `with_parallel_threshold` to all `Datagram`s
    - Add `parallel_threshold` parameter to `ControllerBuilder`
  - Improve performance of `NalgebraBackend`
- Remove `rotation` parameter from `Transducer`, add `rotation` parameter to `Device` instead
- Remove `attenuation` parameter from `Device`
- `SwapSegment` is now enum
- Rename from `as_pascal` to `pascal` and `as_spl` to `spl` in `Amplitude`
- Remove `add_xxx` functions from `Fourier`, `GainSTM`, `FocusSTM`, `Greedy`, `SDP`, `Naive`, `GS`, `GSPAT`, and `LM`
  - User must pass all data in constructor
- Add `Mixer`, `Csv`, and `Custom` modulation

# 24.1.0

- Add `gain::Bessel2`
  - `gain::Bessel` is now deprecated
- Add `with_transform2` for gains
  - `with_transform` is now deprecated

# 24.0.0

- Remove per device ultrasound frequency configuration
- Fix typo: Immidiate -> Immediate
- Fix for capi
  - Add `FocusSTM::stm_sampling_config` and `GainSTM::stm_sampling_config`
  - Add `FPGAState::state`
- Remove unused Result wrapping from group
- Fix slave index in SOEM link
- `Status` does not contain error message now
- Revert Modulation buffer type to `EmitIntensity`

# 23.1.0

- Support `LoopBehavior` and `TransitionMode` in firmware-emulator

# 23.0.1

- Add `rad` and `deg` to `autd3::prelude`
- Add serde feature in autd3-link-soem

# 23.0.0

- Update firmware to v7.0.0
  - Add new segment transition mode
      - SysTime
      - GPIO
      - Ext
  - Return errors more strictly for invalid operations
  - Validate silencer setting only for current segment
  - Allows ultrasound frequency to be changed
    - Add `with_ultrasound_freq` to `AUTD3`
  - Fix configure PulseWidthEncoder operation
- Renaname functions and structs
  - from `ConfigureDebugSettings` to `DebugSettings`
  - from `ConfigurePhaseFilter` to `PhaseFilter`
  - from `ConfigurePulseWidthEncoder` to `PulseWidthEncoder`
  - from `ConfigureSilencer` to `Silencer`
  - from `ConfigureReadsFPGAState` to `ReadsFPGAState`
  - from `ConfigureForceFan` to `ForceFan`
  - from `SamplingConfiguration` to `SamplingConfig`
  - `ChangeGainSegment`, `ChangeModulationSegment`, `ChangeGainSTMSegment` and `ChangeFocusSTMSegment` are merged into `Segment`
  - from `TransducerTest` to `Custom`
  - from `Pascal` to `Pa`
  - from `Rad` to `rad`
  - from `Deg` to `deg`
- Add `EmulateGPIOIn`
- Add `EmissionConstraint::Multiply`
- Make `Controller::group` and `gain::Group` error message more easy to understand
  - Key type now requires `Debug` traits
- Functions that take a frequency as an argument now use `Freq<u32>` or `Freq<f64>` type
- `Sine` and `Square` frequencies is now strictly checked
  - Users should use `with_freq_nearest` intead to bypass the check
- Change Modulation buffer type from `EmitIntensity` to `u8` 
- Remove correction from `EmitIntensity`
  - Users should use `PulseWidthEncoder` instead
- Remove `PhaseRad`
- Remove `SamplingConfig::from_period`
- Add `modulation::Custom`
- Gain and Modulation calculations are parallelized per device
- Fix `STMFocus` range check
- Support Windows on Arm
- Allow thread/process priority configuration in SOEM link
- Test coverage is now 100%
- Remove deprecated functions and structs


# 22.1.0

- Update firmware to v6.1.0
- Add new `ConfigureDebugSettings` datagram
  - `ConfigureDebugOutputIdx` is now deprecated

# 22.0.4

- Add `left_handed` feature

# 22.0.1

- Add `with_loop_behavior` to `Static`

# 22.0.0

- Update firmware to v6.0.0
  - Support segment control
  - Support pulse width encoder control
  - Remove Modulation delay configuration
  - Add phase additive filter
- Remove `ConfigureModDelay`
- Add `ConfigurePhaseFilter`
- Add `with_segment` to `Gain`, `Modulation`, `FocusSTM`, and `GainSTM`
- Add `with_loop_behavior` to `Modulation`, `FocusSTM`, and `GainSTM`
  - Remove `with_start_idx` and `with_finish_idx` from `FocusSTM` and `GainSTM`
- Fix [#8](https://github.com/shinolab/autd3-rs/issues/8): `phase` parameter of `gain::Bessel` and `gain::Focus` have no effect
  - Rename `phase` parameter of `gain::Bessel`, `gain::Focus`, and `gain::Plane` to `phase_offset`
- Extend supported data type of Lightweight mode
- Add `ControllerBuilder::open_with_timeout`
  - Rename `ControllerBuilder::open_with` to `ControllerBuilder::open`
- Remove `with_cache` and `with_transform` from some `Gain`s
- Remove `with_cache`, `with_transform`, and `with_radiation_pressure` from some `Modulation`s
- Support 32-bit float Wav file in `Wav` modulation
- Refactor the whole codebase

# 21.1.0

- Fix [#7](https://github.com/shinolab/autd3-rs/issues/7): `on_lost` and `on_err` callback don't work
- Add `with_err_handler` to `autd3-link-soem::SOEM`
  - `with_on_err` and `with_on_lost` are now deprecated
- Add `phase` parameter to `gain::Focus` and `gain::Bessel`

# 21.0.1

- bit improve `send` perf
- make return value of `send` faithful to doc comment
- move data loading inside `calc` fn of `Wav` and `RawPCM`

# 21.0.0

- Update firmware to v5.1.0
- Refactor the whole codebase

# 20.0.3

- Fix [#5](https://github.com/shinolab/autd3-rs/issues/5): missing 1/4π factor in propagate
- Fix [#6](https://github.com/shinolab/autd3-rs/issues/6): Output of autd3_gain_holo::GS is invalid

# 20.0.1

- Fix for `sync` feature

# 20.0.0

- Update firmware to v5.0.0
  - Add fixed completion steps algorithm of Silencer  
- Fix `T4010A1_AMPLITUDE` value
- Add calculation of index of Modulation and STM from system time in firmware-emulator

# 19.1.0

- Fix [#3](https://github.com/shinolab/autd3-rs/issues/3): Cannot build with sync feature
- Fix [#4](https://github.com/shinolab/autd3-rs/issues/4): autd3-driver@19.0.1 doesn't follow semver

# 19.0.1

- Update firmware to v4.1.2
- `Stop` is now deprecated
  - Users should send `Silencer` and `gain::Null` manually instead of `Stop`
- Fix [#1](https://github.com/shinolab/autd3-rs/issues/1): modulation::Static::with_sampling_config should be omitted
- Fix [#2](https://github.com/shinolab/autd3-rs/issues/2): FocusSTM with points size of 65536 always return STMStartIndexOutOfRange when start_idx is set

# 19.0.0

- Update firmware to v4.1.0
- Change force fan and reads FPGA info APIs
- Add `with_mode` to `modulation::Sine` and `modulation::Square`
  - Fix [#231](https://github.com/shinolab/autd3-old-monorepo/issues/231): Make modulation::Sine more flex configurable
- Fix [#230](https://github.com/shinolab/autd3-old-monorepo/issues/230): Add tests with single_float and use_meter features
- Fix [#238](https://github.com/shinolab/autd3-old-monorepo/issues/238): [C#] Simulator.WithServerIp causes crash

# 18.0.1

- Fix [#237](https://github.com/shinolab/autd3-old-monorepo/issues/237): [C++] USE_SYSTEM_EIGEN option does not work
- Close [#232](https://github.com/shinolab/autd3-old-monorepo/issues/232): Allow to set the time scale of Simulator's autoplay mode

# 18.0.0

- Update firmware to v4.0.1
  - Fix [#228](https://github.com/shinolab/autd3-old-monorepo/issues/228): Debug output index is not cleared by Clear
- Rename `SamplingConfiguration::new_with_*` to `SamplingConfiguration::from_*`
- Rename `EmitIntensity::new_with_*` to `EmitIntensity::with_*`
- Change type of phase member of Drive struct from `float` to `Phase`
- Change `ConfigureDebugOutputIdx` API
  - Fix [#229](https://github.com/shinolab/autd3-old-monorepo/issues/229): Add tests for ConfigureDebugOutoutIdx
- Change `ConfigureModDelay` API
- Change type of frequency of `modulation::Sine` and `Square` to float.
  - Note that the actual output frequency is limited to an integer.
- Fix [#234](https://github.com/shinolab/autd3-old-monorepo/issues/234): cannot use autd3-link-soem with Utf8Error
- Fix [#235](https://github.com/shinolab/autd3-old-monorepo/issues/235): [C++] missing EnumerateAdapters methods for link::SOEM
- Fix [#236](https://github.com/shinolab/autd3-old-monorepo/issues/236): [C++] Invalid error message for out of range STM

# 17.0.3

- Fix [#224](https://github.com/shinolab/autd3-old-monorepo/issues/224): add py.typed for pyautd3
- Fix [#225](https://github.com/shinolab/autd3-old-monorepo/issues/225): Modulation::Sine::with_amp should be with_intensity
- Fix [#226](https://github.com/shinolab/autd3-old-monorepo/issues/226): Cannot use autd_firmware_writer.ps1 if PATH to JDK jlink is set
- Fix [#227](https://github.com/shinolab/autd3-old-monorepo/issues/227): missing ConfigureDebugOutoutIdx in C++/C#/Python
- Make `mod_delay` member in `Transducer` public

# 17.0.2

- Fix [#218](https://github.com/shinolab/autd3-old-monorepo/issues/218): [C++/C#/Python] Transducer does not have tr_idx and dev_idx methods
- Fix [#219](https://github.com/shinolab/autd3-old-monorepo/issues/219): [C++] inconsistent name autd3::gain::holo::AmplitudeConstraint
- Fix [#220](https://github.com/shinolab/autd3-old-monorepo/issues/220): [C#] missing WithIntensity in Modulation.Static
- Fix [#221](https://github.com/shinolab/autd3-old-monorepo/issues/221): [C++] inconsistent api name Focus::new_with_sampling_configuration
- Fix [#222](https://github.com/shinolab/autd3-old-monorepo/issues/222): [C++] inconsistent api name ModulationWithFreqDiv::with_sampling_configuration

# 17.0.1

- Fix [#217](https://github.com/shinolab/autd3-old-monorepo/issues/217): v4.0.0 fpga firmware returns its version as v4.0.1

# 17.0.0

- Update firmware to v4.0.0
  - Remove variable frequency feature
  - Remove filter feature
- Add `EmitIntensity` instead of normalized amplitude
  - Add `with_intensity` to `Gain` instead of `with_amp`
  - Modulation data is now use `EmitIntensity` instead of normalized amplitude
  - `FocusSTM` can now specify intensity instead of `duty_shift`
- Change sampling frequency configuration API
  - Add `with_sampling_config` to `Modulation`
  - Add `new_with_sampling_config` to `GainSTM` and `FocusSTM`
- Change device rotation API
  - Add `with_rotation` to `AUTD3`
- Change silencer API
  - Silecer constructor now takes steps for intensity and phase, respectively
- Update holo gain's `add_focus` methods to take target amplitude instead of normalized amplitude
- Make functions of `Link` async
- Remove `Controller::software_stm`
- Remove `modulation::FIR`
- Remove `modulation::SineLegacy`
- Remove `gain::holo::EVP`
- Remove `Lightweight` options
- `modulation::Square` with out of range duty ratio now return Err
- Purge `link-visualizer`, `backend-cuda`, `backend-arrayfire` crates into other repositories
- Fix [#212](https://github.com/shinolab/autd3-old-monorepo/issues/212): unity-linux and unity-mac packages are not published
- Fix [#213](https://github.com/shinolab/autd3-old-monorepo/issues/213): simulator settings file can not be saved on unity
- Fix [#215](https://github.com/shinolab/autd3-old-monorepo/issues/215): simulator auto_run works only in info tab

# 16.0.3

- Fix [#208](https://github.com/shinolab/autd3-old-monorepo/issues/208): pyautd3-16.0.2-py3-none-linux_armv7l.whl contains aarch64 binary
- Fix [#209](https://github.com/shinolab/autd3-old-monorepo/issues/209): cannot launch simulator from unity

# 16.0.2

- Fix [#203](https://github.com/shinolab/autd3-old-monorepo/issues/203): missing autd-server installer for windows and macos
- Fix [#204](https://github.com/shinolab/autd3-old-monorepo/issues/204): no binary found in linux arm packages

# 16.0.0

- Add `modulation::FIR` for C++/C#/Python
- Add `link::Visualizer` for C++/C#/Python
- Add `link::Simulator::update_geometry`
- Add `Amplitude` and `DutyRatio` for more flexible amplitude specification
- Remove `link::Debug`, `link::Log` and `link::Bundle`
- Remove `link::SOEM::with_log*` methods, add `with_on_err` instead
- Refactor internal code
- Fix [#201](https://github.com/shinolab/autd3-old-monorepo/issues/201): The results of Device::translate_to and Device::rotate_to are wrong
- Fix [#202](https://github.com/shinolab/autd3-old-monorepo/issues/202): Depending on the timing, Link::wait_msg_processed may fall into an infinite loop

# 15.3.1

- Fix [#199](https://github.com/shinolab/autd3-old-monorepo/issues/199): Add Device::translate_to and Device::rotate_to
- Fix [#200](https://github.com/shinolab/autd3-old-monorepo/issues/200): Drive data also updated by notify_link_geometry_updated

# 15.3.0

- Raise minimum supported Python version to 3.10
- Fix [#195](https://github.com/shinolab/autd3-old-monorepo/issues/195): missing setter for speed of sound in Geometry for C++/C#/Python
- Fix [#196](https://github.com/shinolab/autd3-old-monorepo/issues/196): modulation::Fourier must have one component at least
- Fix [#197](https://github.com/shinolab/autd3-old-monorepo/issues/197): Add ability to update Geometry in Simulator
- Fix [#198](https://github.com/shinolab/autd3-old-monorepo/issues/198): [C#] Make Controller::group and Gain::Group can use any type as key

# 15.2.1

- impl `IntoIterator` for `Geometry` and `Device`
- Fix [#194](https://github.com/shinolab/autd3-old-monorepo/issues/194): Cannot use C++ library on macOS 

# 15.2.0

- Merge `GeometryViewer` into Simulator
- Update C++ version to C++20
- Fix [#176](https://github.com/shinolab/autd3-old-monorepo/issues/176): [C++] modulation::Cache can cause errors due to double free
- Fix [#177](https://github.com/shinolab/autd3-old-monorepo/issues/177): [C++] Avoid unnecessary copying in gain::Cache
- Fix [#178](https://github.com/shinolab/autd3-old-monorepo/issues/178): The value of TimerStrategy is not consistent between capi and the original source
- Fix [#164](https://github.com/shinolab/autd3-old-monorepo/issues/164): [Python] Controller.software_stm sometimes stucks
- Fix [#180](https://github.com/shinolab/autd3-old-monorepo/issues/180): [C#] Gain.WithCache and Gain.WIthTransform do not work
- Fix [#181](https://github.com/shinolab/autd3-old-monorepo/issues/181): [C#] Rename BackendCUDA to CUDABackend for consistency
- Fix [#179](https://github.com/shinolab/autd3-old-monorepo/issues/179): [Python] Inconsistent parameter order in TransducerTest.set
- Fix [#184](https://github.com/shinolab/autd3-old-monorepo/issues/184): [C++] Cannot use Gain::with_transform
- Fix [#185](https://github.com/shinolab/autd3-old-monorepo/issues/185): [C++] Cannot use AUTD3_IMPL_WITH_CACHE_GAIN and other similar macros
- Fix [#186](https://github.com/shinolab/autd3-old-monorepo/issues/186): [C++] cannot compile autd3::gain::holo::*
- Fix [#187](https://github.com/shinolab/autd3-old-monorepo/issues/187): [C++] missing with_cache and with_transform for autd3::gain::holo::*
- Fix [#188](https://github.com/shinolab/autd3-old-monorepo/issues/188): [C++] autd3::gain::holo::LM::with_k_max type is invalid
- Fix [#189](https://github.com/shinolab/autd3-old-monorepo/issues/189): [C++] autd3::gain::holo::Greedy cause an SEHException
- Fix [#190](https://github.com/shinolab/autd3-old-monorepo/issues/190): [C++] Modulation::with_transform cause an SEH exception
- Fix [#191](https://github.com/shinolab/autd3-old-monorepo/issues/191): [C++] Transducer::rotation value order is invalid
- Fix [#192](https://github.com/shinolab/autd3-old-monorepo/issues/192): [C++] missing FPGA_CLK_FREQ and FPGA_SUB_CLK_FREQ
- Fix [#182](https://github.com/shinolab/autd3-old-monorepo/issues/182): Cannot use autd3-link-visualizer with python or gpu features
- Fix [#183](https://github.com/shinolab/autd3-old-monorepo/issues/183): Controller::close should also stop output for disabled devices
- Fix [#193](https://github.com/shinolab/autd3-old-monorepo/issues/193): missing Device::enable flag in capi

# 15.1.2

- Fix [#158](https://github.com/shinolab/autd3-old-monorepo/issues/158): Cache doesn't actually do anything
- Fix [#159](https://github.com/shinolab/autd3-old-monorepo/issues/159): `Controller::group` should save device enable flag
- Fix [#163](https://github.com/shinolab/autd3-old-monorepo/issues/163): FirmwareInfo.latest_version still returns "v3.0.0"
- Fix [#165](https://github.com/shinolab/autd3-old-monorepo/issues/165): [Python] missing with_amp methods in Plane
- Fix [#166](https://github.com/shinolab/autd3-old-monorepo/issues/166): There is no way to get modulation size
- Fix [#167](https://github.com/shinolab/autd3-old-monorepo/issues/167): [Python] Gain.with_cache cause an error
- Fix [#168](https://github.com/shinolab/autd3-old-monorepo/issues/168): [Python] GainSTM has from_sampling_frequency_division instead with_sampling_frequency_division
- Fix [#169](https://github.com/shinolab/autd3-old-monorepo/issues/168): Slightly different duty ratios in Legacy and Advanced modes
- Fix [#170](https://github.com/shinolab/autd3-old-monorepo/issues/170): Add sampling period settings for STM in capi
- Fix [#171](https://github.com/shinolab/autd3-old-monorepo/issues/171): [Python] SOEM.enumerate_adapters() cause an error
- Fix [#172](https://github.com/shinolab/autd3-old-monorepo/issues/172): [Python] could not use CUDABackend
- Fix [#173](https://github.com/shinolab/autd3-old-monorepo/issues/173): [Python] cannot use TwinCAT: pyautd3.autd_error.AUTDError: TcAdsDll not found. Please install TwinCAT3
- Fix [#175](https://github.com/shinolab/autd3-old-monorepo/issues/175): rm autd3_model.rs

# 15.1.1

- Fix [#156](https://github.com/shinolab/autd3-old-monorepo/issues/156): Err should be returned when the focus range available is exceeded in FocusSTM
- Fix [#157](https://github.com/shinolab/autd3-old-monorepo/issues/157): Document sample is wrong (AUTD3::DEVICE_WIDTH is not available in pyautd3)
- Add `Controller::software_stm` function to C++, C#, and Python
- Add `Controller::group` function C++, C#, and Python

# 15.1.0

- Add `group` func to `Controller`
- Fix [#149](https://github.com/shinolab/autd3-old-monorepo/issues/149): Simulator's Info should show information for each device
- Fix [#152](https://github.com/shinolab/autd3-old-monorepo/issues/152): [python] Stop does not work
- Fix [#153](https://github.com/shinolab/autd3-old-monorepo/issues/153): [python] cannot run simulator and geometry_viewer examples
- Fix [#154](https://github.com/shinolab/autd3-old-monorepo/issues/154): emulator version is still v3.0.0
- Fix [#155](https://github.com/shinolab/autd3-old-monorepo/issues/155): missing "Filter" section in document

# 15.0.2

- Update firmware to v3.0.2
- Fix [#143](https://github.com/shinolab/autd3-old-monorepo/issues/143): Unify Rust/C++/C#/Python API
- Fix [#147](https://github.com/shinolab/autd3-old-monorepo/issues/147): Remove duplicate resources in unity
- Fix [#148](https://github.com/shinolab/autd3-old-monorepo/issues/148): SoftwareSTM api is complex and difficult to use
- Fix [#150](https://github.com/shinolab/autd3-old-monorepo/issues/150): To remove indirect references, Transducer in C++/C#/Python should own TransducerPtr instead of DevicePtr

# 15.0.1

- Fix [#138](https://github.com/shinolab/autd3-old-monorepo/issues/138): [Unity] can't build because The type or namespace name UnityEditor could not be found

# 15.0.0

- Update firmware to v3.0.1
- `Geometry` is now hierarchical; `Geometry` is a container of `Device`, and `Device` is a container of `Transducer`.
- Remove `autd3-gain-holo::Backend`, add `autd3-gain-holo::LinAlgBackend` instead
- Add `autd3-backend-arrayfire`
- Add `LightweightTwinCATAUTDServer`
- Rename `autd3-traits` to `autd3-derive`
- Rename `autd3-link-monitor` to `autd3-link-visualizer`
- Rename `gain::Grouped` to `gain::Group`, and improve performance
- Add C# documentation
  - Fix [#90](https://github.com/shinolab/autd3-old-monorepo/issues/90): [C#] Poor documentation
- Fix [#135](https://github.com/shinolab/autd3-old-monorepo/issues/135): Compile error of 'autd3-backend-cuda v14.2.2'
- Fix [#137](https://github.com/shinolab/autd3-old-monorepo/issues/137): ninja is needed when build cpp examples on Windows
- Fix [#141](https://github.com/shinolab/autd3-old-monorepo/issues/141): cannot compile with native arm64 linux machine

# 14.2.2

- Add C++ documentation
  - Fix [#89](https://github.com/shinolab/autd3-old-monorepo/issues/89): [C++] Poor documentation
- Fix [#121](https://github.com/shinolab/autd3-old-monorepo/issues/121): Phase parameters of autd3::modulation::Fourier
- Fix [#122](https://github.com/shinolab/autd3-old-monorepo/issues/122): Calling GeometryViewer::run multiple times causes an error
- Fix [#123](https://github.com/shinolab/autd3-old-monorepo/issues/123): impl Default for autd3-link-monitor::PyPlotConfig
- Fix [#124](https://github.com/shinolab/autd3-old-monorepo/issues/124): Python backend of autd3-link-monitor causes indentation errors
- Fix [#125](https://github.com/shinolab/autd3-old-monorepo/issues/125): Grouped without specified Gain for all devices causes an error

# 14.2.1

- Improve `autd3-gain-holo` performance
  - Close [#98](https://github.com/shinolab/autd3-old-monorepo/issues/98): Add benchmarking
- Fix [#118](https://github.com/shinolab/autd3-old-monorepo/issues/118): Cannot compile and test with `single_float` features
- Fix [#119](https://github.com/shinolab/autd3-old-monorepo/issues/119): link-simulator sometimes panic
- Fix [#120](https://github.com/shinolab/autd3-old-monorepo/issues/120): With 9 devices, CUDABackend causes an error with LM algorithm

# 14.2.0

- Add `modulation::Fourier`
  - Fix [#110](https://github.com/shinolab/autd3-old-monorepo/issues/110): Multi-frequency sine modulation? 
- Fix [#111](https://github.com/shinolab/autd3-old-monorepo/issues/111): Add macOS and Linux support for autd3-unity
- Fix [#115](https://github.com/shinolab/autd3-old-monorepo/issues/115): `autd3-geometry-viewer` from git does not work
- Fix [#116](https://github.com/shinolab/autd3-old-monorepo/issues/116): [autd3-unity] cannot launch simulator
- Fix [#117](https://github.com/shinolab/autd3-old-monorepo/issues/117): [autd3-unity] There is no LICENSE.md but LICENSE.txt

# 14.1.0

- Fix [#93](https://github.com/shinolab/autd3-old-monorepo/issues/93): pyautd3 package contains unnecessary dynamic libraries
- Fix [#94](https://github.com/shinolab/autd3-old-monorepo/issues/94): pyautd3 library should clarify its dependence on numpy
- Fix [#108](https://github.com/shinolab/autd3-old-monorepo/issues/108): OsalTimer on macOS causes segmentation fault
- Fix [#109](https://github.com/shinolab/autd3-old-monorepo/issues/109): Add support for linux arm architecture for Raspberry Pi
- Fix [#112](https://github.com/shinolab/autd3-old-monorepo/issues/112): Add gain to Grouped by device group
- Fix [#113](https://github.com/shinolab/autd3-old-monorepo/issues/113): simulator_client example is broken in C++/C#/F#/Python
- Fix [#114](https://github.com/shinolab/autd3-old-monorepo/issues/114): AUTD Server on Windows without npcap installed causes an error
- Add `with_sampling_period` to `Modulation`
- Add `with_period` and `with_sampling_period` to `STM`

# 14.0.1

- Fix [#107](https://github.com/shinolab/autd3-old-monorepo/issues/107): There is no with_sampling_frequency method in FocusSTM and GainSTM in pyautd3
- Add sampling frequency option to `Wav` modulation in capi

# 14.0.0

- Fix [#84](https://github.com/shinolab/autd3-old-monorepo/issues/84): AUTD Server should not require wpcap.dll
- Fix [#85](https://github.com/shinolab/autd3-old-monorepo/issues/85): Dockerfile in doc is broken
- Fix [#86](https://github.com/shinolab/autd3-old-monorepo/issues/86): Remove bindgen dependency from autd3-link-soem
- Fix [#87](https://github.com/shinolab/autd3-old-monorepo/issues/87): Firmware version from Simulator is invalid in macOS
- Fix [#88](https://github.com/shinolab/autd3-old-monorepo/issues/88): [Rust] Poor documentation
- Fix [#92](https://github.com/shinolab/autd3-old-monorepo/issues/92): Support modulation::Radiation in C++/C#/Python
- Fix [#95](https://github.com/shinolab/autd3-old-monorepo/issues/95): Poor typing in pyautd3
- Fix [#96](https://github.com/shinolab/autd3-old-monorepo/issues/96): sudo pip is not recommended
- Fix [#97](https://github.com/shinolab/autd3-old-monorepo/issues/97): Can AMS Net Id be displayed on terminal?
- Fix [#99](https://github.com/shinolab/autd3-old-monorepo/issues/99): Add gain::Grouped support in lightweight mode
- Fix [#100](https://github.com/shinolab/autd3-old-monorepo/issues/100): AUTD Server application should show License
- Fix [#102](https://github.com/shinolab/autd3-old-monorepo/issues/102): error message when given an Interface name and no AUTD3 device is not found is a bit strange
- Fix [#103](https://github.com/shinolab/autd3-old-monorepo/issues/103): warning: variable does not need to be mutable in tests
- Fix [#104](https://github.com/shinolab/autd3-old-monorepo/issues/104): dependency on autd3-protobuf should be optional because it requires additional libraries to build and is not necessary for basic usage
- Fix [#105](https://github.com/shinolab/autd3-old-monorepo/issues/105): pyautd3 cannot be used on macOS
- Fix [#106](https://github.com/shinolab/autd3-old-monorepo/issues/106): tuple of Clear and Synchronize can be sent, but it must be not allowed
- Add plotters backend for `autd3-link-monitor` and make it default

# 13.0.0

- Remove `SinePressure`, add `RadiationPressure` instead
- Adopt gRPC for more stable remote communication
- Integrated SOEMAUTDServer/TwinCATAUTDServer/simulator into AUTD server app
- Send `Clear` and `Synchronize` in `open` automatically

# 12.3.1

- Fix [#82](https://github.com/shinolab/autd3-old-monorepo/issues/82)
- Fix [#83](https://github.com/shinolab/autd3-old-monorepo/issues/83)

# 12.3.0

- Fix [#81](https://github.com/shinolab/autd3-old-monorepo/issues/81)
  - Raise minimum supported Python version to 3.9

# 12.2.0

- Add `send_async`
- Add `software_stm`
- Fix [#80](https://github.com/shinolab/autd3-old-monorepo/issues/80)

# 12.1.1

- Fix [#78](https://github.com/shinolab/autd3-old-monorepo/issues/78)
- Fix [#79](https://github.com/shinolab/autd3-old-monorepo/issues/79)

# 12.1.0

- Fix [#76](https://github.com/shinolab/autd3-old-monorepo/issues/76)
- Fix [#77](https://github.com/shinolab/autd3-old-monorepo/issues/77)

# 12.0.0

- Fix [#75](https://github.com/shinolab/autd3-old-monorepo/issues/75)

# 11.2.0

- Fix [#69](https://github.com/shinolab/autd3-old-monorepo/issues/69)
- Fix [#70](https://github.com/shinolab/autd3-old-monorepo/issues/70)
- Add `Bundle` link
- Add `Monitor` link
- Add `FIR` modulation
- Add `Transform` modulation
- Add `RawPCM` modulation
- Fix fluctuation when moving slice in simulator

# 11.1.0

- Fix [#68](https://github.com/shinolab/autd3-old-monorepo/issues/68)
- Improve Simulator stability

# 11.0.2

- Fix [#74](https://github.com/shinolab/autd3-old-monorepo/issues/74)

# 11.0.1

- minor fix

# 11.0.0

- Fix [#63](https://github.com/shinolab/autd3-old-monorepo/issues/63)
- Fix [#64](https://github.com/shinolab/autd3-old-monorepo/issues/64)
- Fix [#65](https://github.com/shinolab/autd3-old-monorepo/issues/65)
- Fix [#67](https://github.com/shinolab/autd3-old-monorepo/issues/67)
- Fix [#71](https://github.com/shinolab/autd3-old-monorepo/issues/71)
- Fix [#72](https://github.com/shinolab/autd3-old-monorepo/issues/72)

# 10.0.0

- Fix [#62](https://github.com/shinolab/autd3-old-monorepo/issues/62)
- All codes are rewritten in Rust

# 9.0.1

- Minimize dependence on boost
- Add `link::RemoteSimulator` to capi

# 9.0.0

- AUTD Simulator can now be accessed over the network
  - Add `link::RemoteSimulator`
- Add logging and timeout options for all links 

# 8.5.0

- (internal) Refactor some modules and adopt Boost library

# 8.4.1

- Fix [#60](https://github.com/shinolab/autd3-old-monorepo/issues/60)
- Fix [#61](https://github.com/shinolab/autd3-old-monorepo/issues/61)

# 8.4.0

- Add default timeout option to `Link`

# 8.3.0

- Fix some minor bugs
- Add `timer_strategy` option for `link::SOEM`
  - `high_precision` is now deprecated
- (internal) Refactor some modules

# 8.2.0

- Move `Controller::set_sound_speed_from_temp` into `Geometry`
- Make `modulation::LPF` generic
- Add `modulation::Transform`
- Rename Normal mode to Advanced mode
- Rename `gain::holo::EVD` to `gain::holo::EVP`
- Remove `<<` operators
- Remove `ack_check_timeout` option, add `timeout` parameter to `send` function instead

# 8.1.2

- Fix [#59](https://github.com/shinolab/autd3-old-monorepo/issues/59)

# 8.1.1

- Fix [#58](https://github.com/shinolab/autd3-old-monorepo/issues/58)

# 8.1.0

- Introduce Semantic versioning
- Add thermal sensor option
- Add vivado lab edition supoprt for firmware update
- Add `link::Log`
- Add geometry transformation methods
- Remove async send methods
- Change `Controller::open` API
  - Add `Geometry::Builder` to create geometry

# 2.8.0

- Suppress midrange frequency noise

# 2.7.6

- Fix [#57](https://github.com/shinolab/autd3-old-monorepo/issues/57)

# 2.7.5

- Fix [#37](https://github.com/shinolab/autd3-old-monorepo/issues/37)

# 2.7.4

- Fix [#55](https://github.com/shinolab/autd3-old-monorepo/issues/55)
- Fix [#56](https://github.com/shinolab/autd3-old-monorepo/issues/56)

# 2.7.3

- Remove parameter from `FocusSTM`, `GainSTM`, and `gain::Grouped` constructor
- Remove `Driver` to drive old firmware
- Fix [#54](https://github.com/shinolab/autd3-old-monorepo/issues/54)

# 2.7.2

- Fix [#52](https://github.com/shinolab/autd3-old-monorepo/issues/52)
- Fix [#53](https://github.com/shinolab/autd3-old-monorepo/issues/53)

# 2.7.1

- Fix [#51](https://github.com/shinolab/autd3-old-monorepo/issues/51)
- Add `USE_SINGLE_FLOAT` option
- [Unity] Unity API now uses `float` instead of `double`

# 2.7.0 

- Fix [#50](https://github.com/shinolab/autd3-old-monorepo/issues/50)
- Add `start_idx` and `finish_idx` to `STM`

# 2.6.8

- Fix [#49](https://github.com/shinolab/autd3-old-monorepo/issues/49)
- Improve `Holo` gain performance

# 2.6.7

- Change `Controller::_send_interval` to 1ms by default
- Improve Simulator stability
- Add "Auto play" option for Simulator

# 2.6.6

- Rename `PointSTM` to `FocusSTM`
- Add `AUTDSetSoundSpeedFromTemp` in capi
- (internal) refactor to improve readability

# 2.6.5

- Flatten `Geometry`
- Fix [#48](https://github.com/shinolab/autd3-old-monorepo/issues/48)

# 2.6.4

- Fix [#46](https://github.com/shinolab/autd3-old-monorepo/issues/46)
- Fix [#47](https://github.com/shinolab/autd3-old-monorepo/issues/47)

# 2.6.3

- Change sound speed configuration API
- Functions that can fail now return false instead of an exception
  - C API has also been changed
- Fix [#45](https://github.com/shinolab/autd3-old-monorepo/issues/45)

# 2.6.2

- Remove `Controller::check_trials`, add `Controllse::set_ack_check_timeout` instead
- Add `driver::Driver` class to drive old firmware
- Change `gain::TransducerTest` API

# 2.6.1

- Fix [#44](https://github.com/shinolab/autd3-old-monorepo/issues/44)

# 2.6.0

- Fix [#43](https://github.com/shinolab/autd3-old-monorepo/issues/43)
- `MOD_SAMPLING_FREQ_DIV_MIN`, `POINT_STM_SAMPLING_FREQ_DIV_MIN`, `GAIN_STM_SAMPLING_FREQ_DIV_MIN`, `GAIN_STM_LEGACY_SAMPLING_FREQ_DIV_MIN`, and `SILENCER_CYCLE_MIN` are halved

# 2.5.2

- Fix [#37](https://github.com/shinolab/autd3-old-monorepo/issues/37)
- Fix [#42](https://github.com/shinolab/autd3-old-monorepo/issues/42)
- Change phase unit to radian
- Add stream operator to `Controller`
- Add `Controller::send_async` and `autd3::async` to send data asynchronously
- `Controller::synchronize()`, `Controller::update_flag()`, `Controller::clear()`, and `Controller::stop()` functions are now deprecated
  - Send `autd3::synchronize`, `autd3::update_flag`, `autd3::clear`, and `autd3::stop` instead

# 2.5.1

- Fix [#38](https://github.com/shinolab/autd3-old-monorepo/issues/38)
- Fix [#39](https://github.com/shinolab/autd3-old-monorepo/issues/39)
- Fix [#40](https://github.com/shinolab/autd3-old-monorepo/issues/40)
- Fix [#41](https://github.com/shinolab/autd3-old-monorepo/issues/41)
- Add simulator for Unity

# 2.5.0

- Rename `AUTDServer` to `TwinCATAUTDServer`
- Add `SOEMAUTDServer` and `link::RemoteSOEM`
- Add Windows arm support
- Fix [#36](https://github.com/shinolab/autd3-old-monorepo/issues/36)
- Add log settings in SOEM Link CAPI
- Remove `port` and `ip` settings from Simulator

# 2.4.5

- Change unit of sound speed from m/s to mm/s
- Add `use_meter` and `use_left_handed` options to `Simulator`
- Change `Holo` constraint API

# 2.4.4

- Change default EtherCAT interval 500us to 1ms
- Improve `link::SOEM` performance
- [AUTD3Sharp] Change API to improve consistency with the C++ version
- [pyautd3] Change API to improve consistency with the C++ version

# 2.4.3

- Embed font into GeometryViewer and Simulator
- Embed model into GeometryViewer

# 2.4.2

- win-x86 is no more supported
- Fix [#30](https://github.com/shinolab/autd3-old-monorepo/issues/30)
- Fix minor bugs

# 2.4.1

- Add `extra::simulator::Simulator`
- Rename `link::Emulator` to `link::Simulator`

# 2.4.0

- Add `GeometryViewer`
- Improve performance of `link::SOEM` on Windows
- Update maximum pattern size of `GaimSTM` in legacy mode to 2048
- Add `link::Bundle` and `link::Debug`
- Add `extra::firmware-emulator`
- Add `SoftwareSTM`
- Add `gain::TransducerTest`
- Add `modulation::LPF`
- Add `ArrayFireBackend` (experimental)
- Fix [#25](https://github.com/shinolab/autd3-old-monorepo/issues/25), [#26](https://github.com/shinolab/autd3-old-monorepo/issues/26)
- Update firmware to v2.4

# 2.3.1

- Remove the first argument (`Geometry&`) of a `link::Emulator` constructor
- Remove the first argument (interface name) and the second argument (number of devices) of a `link::SOEM` constructor
- (internal) `link::open` now requires `Geometry&`

# 2.3.0

- Add `SyncMode` setting to `link::SOEM` to address #20
  - Remove `cycle_ticks` parameter, add `send_cycle` and `sync0_cycle` instead
- `MOD_SAMPLING_FREQ_DIV_MIN`, `STM_SAMPLING_FREQ_DIV_MIN`, and `SILENCER_CYCLE_MIN` are halved
- Update firmware to v2.3

# 2.2.2

- Change the whole API; this library is no more using a template to change the Transducer mode
- Add `gain::holo::LSSGreedy` and `gain::holo::APO`

# 2.2.1

- Remove a `check_ack` flag, and add a `check_trials` parameter instead
  - `check_ack = true` equals to `check_trials = 50` and `check_ack = false`
    equals to `check_trials = 0`
- Add `send_interval` parameter
  - default is 1
- Remove a `sound_speed` parameter from `AUTDGetWavelength` in C-API
- Remove `GaussNewton` and `GradientDescent`

# 2.2.0

- Remove `AUTDSendHeader`, `AUTDSendBody`, and `AUTDSend` in C-API, which are now merged into `AUTDSend`
- Remove `cycle_ticks` parameters in `link::TwinCAT` and `link::RemoteTwinCAT`,
  which are no more required
- (internal) Remove `cycle_ticks` method from `link::Link`
- Update firmware to v2.2

# 2.1.0

- Rename `Controller` to `ControllerX`, and `Controller` is now an alias of `ControllerX<LegacyTransducer>`
- Add `NormalPhaseTransducer`
- Fix `SineLegacy`
- Fix firmware version when using v1.x firmware
- Add `Mode` configuration in `GainSTM`
- Add `mod_delay` configuration in `Transducer`
- Update firmware to v2.1

# 2.0.3

- Fix `AUTDSetSoundSpeed` in C-API

# 2.0.2

- Add `DynamicTransducer` for C-API
- Remove legacy C-API library
- Change `AmplitudeConstraint` API in Holo Gain in C-API
- Fix `wavelength` and `wavenumber` of `NormalTransducer`

# 2.0.1

- Fix C-API
- Add `objective` parameter to `gain::holo::Greedy`
- Fix a bug in sending modulation and gain separately
- Change Silencer API

