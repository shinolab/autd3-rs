use std::time::Duration;

use crate::{
    local::{
        error_handler::{ErrHandler, Status},
        SyncMode,
    },
    SOEM,
};

use autd3_driver::{
    derive::*, error::AUTDInternalError, link::LinkBuilder, timer_strategy::TimerStrategy,
};

#[derive(Builder)]
#[no_const]
pub struct SOEMBuilder {
    #[getset]
    pub(crate) buf_size: usize,
    #[getset]
    pub(crate) timer_strategy: TimerStrategy,
    #[getset]
    pub(crate) sync_mode: SyncMode,
    #[getset]
    pub(crate) ifname: String,
    #[getset]
    pub(crate) state_check_interval: std::time::Duration,
    #[getset]
    pub(crate) timeout: std::time::Duration,
    #[getset]
    pub(crate) sync0_cycle: u64,
    #[getset]
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

    /// Set callback function when error occurred
    pub fn with_err_handler(
        self,
        err_handler: impl Fn(usize, Status) + Send + Sync + 'static,
    ) -> Self {
        Self {
            err_handler: Some(Box::new(err_handler)),
            ..self
        }
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
