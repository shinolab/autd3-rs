use std::time::Duration;

use crate::{
    local::{
        error_handler::{ErrHandler, Status},
        SyncMode,
    },
    SOEM,
};

use autd3_driver::{error::AUTDInternalError, link::LinkBuilder, timer_strategy::TimerStrategy};

pub struct SOEMBuilder {
    pub(crate) buf_size: usize,
    pub(crate) timer_strategy: TimerStrategy,
    pub(crate) sync_mode: SyncMode,
    pub(crate) ifname: String,
    pub(crate) state_check_interval: std::time::Duration,
    pub(crate) timeout: std::time::Duration,
    pub(crate) sync0_cycle: u64,
    pub(crate) send_cycle: u64,
    pub(crate) err_handler: Option<ErrHandler>,
}

impl Default for SOEMBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SOEMBuilder {
    pub const fn new() -> Self {
        SOEMBuilder {
            buf_size: 32,
            timer_strategy: TimerStrategy::Sleep,
            sync_mode: SyncMode::FreeRun,
            ifname: String::new(),
            state_check_interval: Duration::from_millis(100),
            timeout: Duration::from_millis(20),
            sync0_cycle: 2,
            send_cycle: 2,
            err_handler: None,
        }
    }

    /// Set sync0 cycle (the unit is 500us)
    pub fn with_sync0_cycle(self, sync0_cycle: u64) -> Self {
        Self {
            sync0_cycle,
            ..self
        }
    }

    /// Set send cycle (the unit is 500us)
    pub fn with_send_cycle(self, send_cycle: u64) -> Self {
        Self { send_cycle, ..self }
    }

    /// Set send buffer size
    pub fn with_buf_size(self, buf_size: usize) -> Self {
        Self { buf_size, ..self }
    }

    /// Set timer strategy
    pub fn with_timer_strategy(self, timer_strategy: TimerStrategy) -> Self {
        Self {
            timer_strategy,
            ..self
        }
    }

    /// Set sync mode
    ///
    /// See [Beckhoff's site](https://infosys.beckhoff.com/content/1033/ethercatsystem/2469122443.html) for more details.
    pub fn with_sync_mode(self, sync_mode: SyncMode) -> Self {
        Self { sync_mode, ..self }
    }

    /// Set network interface name
    ///
    /// If empty, this link will automatically find the network interface that is connected to AUTD3 devices.
    ///
    pub fn with_ifname<S: Into<String>>(self, ifname: S) -> Self {
        Self {
            ifname: ifname.into(),
            ..self
        }
    }

    /// Set state check interval
    pub fn with_state_check_interval(self, state_check_interval: Duration) -> Self {
        Self {
            state_check_interval,
            ..self
        }
    }

    /// Set callback function when error occurred
    pub fn with_err_handler<F: 'static + Fn(usize, Status) + Send + Sync>(
        self,
        err_handler: F,
    ) -> Self {
        Self {
            err_handler: Some(Box::new(err_handler)),
            ..self
        }
    }

    /// Set timeout
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl LinkBuilder for SOEMBuilder {
    type L = SOEM;

    async fn open(
        self,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self::L, AUTDInternalError> {
        Self::L::open(self, geometry).await
    }
}
