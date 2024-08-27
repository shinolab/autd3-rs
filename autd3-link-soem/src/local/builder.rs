use std::{
    num::{NonZeroU64, NonZeroUsize},
    time::Duration,
};

use super::{
    error_handler::{ErrHandler, Status},
    timer_strategy::TimerStrategy,
    SyncMode, SOEM,
};

use autd3_driver::{derive::*, link::LinkBuilder};

use thread_priority::ThreadPriority;

#[derive(Builder)]
pub struct SOEMBuilder {
    #[get]
    #[set]
    pub(crate) buf_size: NonZeroUsize,
    #[get]
    #[set]
    pub(crate) timer_strategy: TimerStrategy,
    #[get]
    #[set]
    pub(crate) sync_mode: SyncMode,
    #[get(ref)]
    #[set(into)]
    pub(crate) ifname: String,
    #[get]
    #[set]
    pub(crate) state_check_interval: std::time::Duration,
    #[get]
    #[set]
    pub(crate) timeout: std::time::Duration,
    #[get]
    #[set]
    pub(crate) sync0_cycle: NonZeroU64,
    #[get]
    #[set]
    pub(crate) send_cycle: NonZeroU64,
    #[get]
    #[set]
    pub(crate) thread_priority: ThreadPriority,
    #[cfg(target_os = "windows")]
    #[get]
    #[set]
    pub(crate) process_priority: super::ProcessPriority,
    pub(crate) err_handler: Option<ErrHandler>,
    #[get]
    #[set]
    pub(crate) sync_tolerance: std::time::Duration,
    #[get]
    #[set]
    pub(crate) sync_timeout: std::time::Duration,
}

impl Default for SOEMBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SOEMBuilder {
    pub fn new() -> Self {
        SOEMBuilder {
            buf_size: NonZeroUsize::new(32).unwrap(),
            timer_strategy: TimerStrategy::Sleep,
            sync_mode: SyncMode::DC,
            ifname: String::new(),
            state_check_interval: Duration::from_millis(100),
            timeout: Duration::from_millis(20),
            sync0_cycle: NonZeroU64::new(2).unwrap(),
            send_cycle: NonZeroU64::new(2).unwrap(),
            thread_priority: ThreadPriority::Max,
            #[cfg(target_os = "windows")]
            process_priority: super::ProcessPriority::High,
            err_handler: None,
            sync_tolerance: std::time::Duration::from_micros(1),
            sync_timeout: std::time::Duration::from_secs(10),
        }
    }

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
