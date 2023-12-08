/*
 * File: error.rs
 * Project: cpu
 * Created Date: 02/05/2022
 * Author: Shun Suzuki
 * -----
 * Last Modified: 07/12/2023
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2022-2023 Shun Suzuki. All rights reserved.
 *
 */

use thiserror::Error;

use crate::{defined::float, fpga::*, operation::GainSTMMode};

#[derive(Error, Debug, PartialEq)]
pub enum AUTDInternalError {
    #[error(
        "Modulation buffer size ({0}) is out of range ([{}, {}])",
        MOD_BUF_SIZE_MIN,
        MOD_BUF_SIZE_MAX
    )]
    ModulationSizeOutOfRange(usize),

    #[error(
        "Silencer step ({0}) is out of range ([{}, {}])",
        SILENCER_STEP_MIN,
        SILENCER_STEP_MAX
    )]
    SilencerStepOutOfRange(u16),

    #[error("Sampling frequency division ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqDivOutOfRange(u32, u32, u32),
    #[error("Sampling frequency ({0}) is out of range ([{1}, {2}])")]
    SamplingFreqOutOfRange(float, float, float),
    #[error("Sampling period ({0} ns) is out of range ([{1}, {2}])")]
    SamplingPeriodOutOfRange(u128, u128, u128),

    #[error("STM frequency ({1} Hz, size={0}) is out of range ([{2}, {3}])")]
    STMFreqOutOfRange(usize, float, float, float),
    #[error("STM period ({1} ns, size={0}) is out of range ([{2}, {3}])")]
    STMPeriodOutOfRange(usize, u128, usize, usize),

    #[error("STM index is out of range")]
    STMStartIndexOutOfRange,
    #[error("STM finish is out of range")]
    STMFinishIndexOutOfRange,
    #[error(
        "FocusSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        FOCUS_STM_BUF_SIZE_MAX
    )]
    FocusSTMPointSizeOutOfRange(usize),
    #[error(
        "Point ({0}, {1}, {2}) is out of range. Each parameter must be in [{}, {}].",
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_LOWER as float,
        FOCUS_STM_FIXED_NUM_UNIT * FOCUS_STM_FIXED_NUM_UPPER as float,
    )]
    FocusSTMPointOutOfRange(float, float, float),
    #[error(
        "GainSTM size ({0}) is out of range ([{}, {}])",
        STM_BUF_SIZE_MIN,
        GAIN_STM_BUF_SIZE_MAX
    )]
    GainSTMSizeOutOfRange(usize),

    #[error("GainSTMMode ({0:?}) is not supported")]
    GainSTMModeNotSupported(GainSTMMode),

    #[error("{0}")]
    ModulationError(String),
    #[error("{0}")]
    GainError(String),
    #[error("{0}")]
    LinkError(String),

    #[error("{0}")]
    NotSupported(String),

    #[error("Link is closed")]
    LinkClosed,

    #[error("Failed to create timer")]
    TimerCreationFailed,
    #[error("Failed to delete timer")]
    TimerDeleteFailed,

    #[cfg(target_os = "windows")]
    #[error("{0}")]
    WindowsError(#[from] windows::core::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn freq_div_out_of_range() {
        let err = AUTDInternalError::SamplingFreqDivOutOfRange(
            1,
            SAMPLING_FREQ_DIV_MIN,
            SAMPLING_FREQ_DIV_MAX,
        );
        assert!(err.source().is_none());
        assert_eq!(
            format!("{}", err),
            "Sampling frequency division (1) is out of range ([512, 4294967295])"
        );
        assert_eq!(
            format!("{:?}", err),
            "SamplingFreqDivOutOfRange(1, 512, 4294967295)"
        );

        let err = AUTDInternalError::SamplingFreqDivOutOfRange(
            1,
            SAMPLING_FREQ_DIV_MIN,
            SAMPLING_FREQ_DIV_MAX,
        );
        assert_eq!(
            err,
            AUTDInternalError::SamplingFreqDivOutOfRange(
                1,
                SAMPLING_FREQ_DIV_MIN,
                SAMPLING_FREQ_DIV_MAX,
            )
        );
        assert_ne!(
            err,
            AUTDInternalError::SamplingFreqDivOutOfRange(
                2,
                SAMPLING_FREQ_DIV_MIN,
                SAMPLING_FREQ_DIV_MAX,
            )
        );
    }

    #[test]
    fn stm_start_index_out_of_range() {
        let err = AUTDInternalError::STMStartIndexOutOfRange;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "STM index is out of range");
        assert_eq!(format!("{:?}", err), "STMStartIndexOutOfRange");

        let err = AUTDInternalError::STMStartIndexOutOfRange;
        assert_eq!(err, AUTDInternalError::STMStartIndexOutOfRange);
    }

    #[test]
    fn stm_finish_index_out_of_range() {
        let err = AUTDInternalError::STMFinishIndexOutOfRange;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "STM finish is out of range");
        assert_eq!(format!("{:?}", err), "STMFinishIndexOutOfRange");

        let err = AUTDInternalError::STMFinishIndexOutOfRange;
        assert_eq!(err, AUTDInternalError::STMFinishIndexOutOfRange);
    }

    #[test]
    fn focus_stm_point_size_out_of_range() {
        let err = AUTDInternalError::FocusSTMPointSizeOutOfRange(1);
        assert!(err.source().is_none());
        assert_eq!(
            format!("{}", err),
            "FocusSTM size (1) is out of range ([2, 65536])"
        );
        assert_eq!(format!("{:?}", err), "FocusSTMPointSizeOutOfRange(1)");

        let err = AUTDInternalError::FocusSTMPointSizeOutOfRange(1);
        assert_eq!(err, AUTDInternalError::FocusSTMPointSizeOutOfRange(1));
        assert_ne!(err, AUTDInternalError::FocusSTMPointSizeOutOfRange(2));
    }

    #[test]
    fn focus_stm_point_out_of_range() {
        let err = AUTDInternalError::FocusSTMPointOutOfRange(1.0, 2.0, 3.0);
        assert!(err.source().is_none());
        if cfg!(feature = "use_meter") {
            assert_eq!(
                format!("{}", err),
                "Point (1, 2, 3) is out of range. Each parameter must be in [-3.2768, 3.276775]."
            );
        } else {
            assert_eq!(
                format!("{}", err),
                "Point (1, 2, 3) is out of range. Each parameter must be in [-3276.8, 3276.775]."
            );
        }
        assert_eq!(
            format!("{:?}", err),
            "FocusSTMPointOutOfRange(1.0, 2.0, 3.0)"
        );

        let err = AUTDInternalError::FocusSTMPointOutOfRange(1.0, 2.0, 3.0);
        assert_eq!(
            err,
            AUTDInternalError::FocusSTMPointOutOfRange(1.0, 2.0, 3.0)
        );
        assert_ne!(
            err,
            AUTDInternalError::FocusSTMPointOutOfRange(1.0, 2.0, 4.0)
        );
    }

    #[test]
    fn gain_stm_size_out_of_range() {
        let err = AUTDInternalError::GainSTMSizeOutOfRange(1);
        assert!(err.source().is_none());
        assert_eq!(
            format!("{}", err),
            "GainSTM size (1) is out of range ([2, 2048])"
        );
        assert_eq!(format!("{:?}", err), "GainSTMSizeOutOfRange(1)");

        let err = AUTDInternalError::GainSTMSizeOutOfRange(1);
        assert_eq!(err, AUTDInternalError::GainSTMSizeOutOfRange(1));
        assert_ne!(err, AUTDInternalError::GainSTMSizeOutOfRange(2));
    }

    #[test]
    fn silencer_out_of_range() {
        let err = AUTDInternalError::SilencerStepOutOfRange(1);
        assert!(err.source().is_none());
        assert_eq!(
            format!("{}", err),
            "Silencer step (1) is out of range ([1, 65535])"
        );
        assert_eq!(format!("{:?}", err), "SilencerStepOutOfRange(1)");

        let err = AUTDInternalError::SilencerStepOutOfRange(1);
        assert_eq!(err, AUTDInternalError::SilencerStepOutOfRange(1));
        assert_ne!(err, AUTDInternalError::SilencerStepOutOfRange(2));
    }

    #[test]
    fn gain_stm_mode_not_supported() {
        let err = AUTDInternalError::GainSTMModeNotSupported(GainSTMMode::PhaseIntensityFull);
        assert!(err.source().is_none());
        assert_eq!(
            format!("{}", err),
            "GainSTMMode (PhaseIntensityFull) is not supported"
        );
        assert_eq!(
            format!("{:?}", err),
            "GainSTMModeNotSupported(PhaseIntensityFull)"
        );

        let err = AUTDInternalError::GainSTMModeNotSupported(GainSTMMode::PhaseIntensityFull);
        assert_eq!(
            err,
            AUTDInternalError::GainSTMModeNotSupported(GainSTMMode::PhaseIntensityFull)
        );
        assert_ne!(
            err,
            AUTDInternalError::GainSTMModeNotSupported(GainSTMMode::PhaseFull)
        );
    }

    #[test]
    fn modulation_error() {
        let err = AUTDInternalError::ModulationError("error".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "error");
        assert_eq!(format!("{:?}", err), "ModulationError(\"error\")");

        let err = AUTDInternalError::ModulationError("error".to_string());
        assert_eq!(err, AUTDInternalError::ModulationError("error".to_string()));
        assert_ne!(
            err,
            AUTDInternalError::ModulationError("error2".to_string())
        );
    }

    #[test]
    fn gain_error() {
        let err = AUTDInternalError::GainError("error".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "error");
        assert_eq!(format!("{:?}", err), "GainError(\"error\")");

        let err = AUTDInternalError::GainError("error".to_string());
        assert_eq!(err, AUTDInternalError::GainError("error".to_string()));
        assert_ne!(err, AUTDInternalError::GainError("error2".to_string()));
    }

    #[test]
    fn link_error() {
        let err = AUTDInternalError::LinkError("error".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "error");
        assert_eq!(format!("{:?}", err), "LinkError(\"error\")");

        let err = AUTDInternalError::LinkError("error".to_string());
        assert_eq!(err, AUTDInternalError::LinkError("error".to_string()));
        assert_ne!(err, AUTDInternalError::LinkError("error2".to_string()));
    }

    #[test]
    fn not_supported() {
        let err = AUTDInternalError::NotSupported("error".to_string());
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "error");
        assert_eq!(format!("{:?}", err), "NotSupported(\"error\")");

        let err = AUTDInternalError::NotSupported("error".to_string());
        assert_eq!(err, AUTDInternalError::NotSupported("error".to_string()));
        assert_ne!(err, AUTDInternalError::NotSupported("error2".to_string()));
    }

    #[test]
    fn link_closed() {
        let err = AUTDInternalError::LinkClosed;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Link is closed");
        assert_eq!(format!("{:?}", err), "LinkClosed");

        let err = AUTDInternalError::LinkClosed;
        assert_eq!(err, AUTDInternalError::LinkClosed);
    }

    #[test]
    fn timer_creation_failed() {
        let err = AUTDInternalError::TimerCreationFailed;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Failed to create timer");
        assert_eq!(format!("{:?}", err), "TimerCreationFailed");

        let err = AUTDInternalError::TimerCreationFailed;
        assert_eq!(err, AUTDInternalError::TimerCreationFailed);
    }

    #[test]
    fn timer_delete_failed() {
        let err = AUTDInternalError::TimerDeleteFailed;
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "Failed to delete timer");
        assert_eq!(format!("{:?}", err), "TimerDeleteFailed");

        let err = AUTDInternalError::TimerDeleteFailed;
        assert_eq!(err, AUTDInternalError::TimerDeleteFailed);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn windows_error() {
        let win_err =
            windows::core::Error::new(windows::core::HRESULT(1), windows::core::HSTRING::new());
        let err = AUTDInternalError::from(win_err.clone());
        assert_eq!(
            err.source()
                .and_then(|e| e.downcast_ref::<windows::core::Error>()),
            Some(&win_err)
        );
        assert_eq!(format!("{}", err), format!("{}", win_err));
        assert_eq!(format!("{:?}", err), format!("WindowsError({:?})", win_err));

        let err = AUTDInternalError::from(win_err.clone());
        assert_eq!(err, AUTDInternalError::from(win_err));
        assert_ne!(
            err,
            AUTDInternalError::from(windows::core::Error::new(
                windows::core::HRESULT(2),
                windows::core::HSTRING::new(),
            ))
        );
    }
}
