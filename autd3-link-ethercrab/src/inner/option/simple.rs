use std::time::Duration;

use core_affinity::CoreId;
use ethercrab::{MainDeviceConfig, RetryBehaviour, Timeouts, subdevice_group::DcConfiguration};
use thread_priority::{ThreadBuilder, ThreadPriority};

use super::EtherCrabOptionFull;

#[derive(Clone, Debug)]
pub struct EtherCrabOption {
    pub ifname: Option<String>,
    pub buf_size: usize,
    pub state_check_period: Duration,
    pub sync0_period: Duration,
    pub sync_tolerance: Duration,
    pub sync_timeout: Duration,
    pub tx_rx_thread_builder: ThreadBuilder,
    pub tx_rx_thread_core: Option<CoreId>,
    pub main_thread_builder: ThreadBuilder,
    pub main_thread_core: Option<CoreId>,
}

impl Default for EtherCrabOption {
    fn default() -> Self {
        Self {
            ifname: None,
            buf_size: 32,
            state_check_period: Duration::from_millis(100),
            sync0_period: Duration::from_millis(2),
            sync_tolerance: Duration::from_micros(1),
            sync_timeout: Duration::from_secs(20),
            #[cfg(target_os = "windows")]
            tx_rx_thread_builder: ThreadBuilder::default().name("tx-rx-thread").priority(
                ThreadPriority::Os(thread_priority::ThreadPriorityOsValue::from(
                    thread_priority::WinAPIThreadPriority::TimeCritical,
                )),
            ),
            #[cfg(not(target_os = "windows"))]
            tx_rx_thread_builder: ThreadBuilder::default()
                .name("tx-rx-thread")
                .priority(
                    thread_priority::ThreadPriorityValue::try_from(99)
                        .map_or_else(|_| ThreadPriority::Max, ThreadPriority::Crossplatform),
                )
                .policy(thread_priority::ThreadSchedulePolicy::Realtime(
                    thread_priority::RealtimeThreadSchedulePolicy::Fifo,
                )),
            #[cfg(target_os = "windows")]
            main_thread_builder: ThreadBuilder::default().name("main-thread").priority(
                ThreadPriority::Os(thread_priority::ThreadPriorityOsValue::from(
                    thread_priority::WinAPIThreadPriority::TimeCritical,
                )),
            ),
            #[cfg(not(target_os = "windows"))]
            main_thread_builder: ThreadBuilder::default()
                .name("main-thread")
                .priority(
                    thread_priority::ThreadPriorityValue::try_from(99)
                        .map_or_else(|_| ThreadPriority::Max, ThreadPriority::Crossplatform),
                )
                .policy(thread_priority::ThreadSchedulePolicy::Realtime(
                    thread_priority::RealtimeThreadSchedulePolicy::Fifo,
                )),
            main_thread_core: None,
            tx_rx_thread_core: None,
        }
    }
}

impl From<EtherCrabOption> for EtherCrabOptionFull {
    fn from(opt: EtherCrabOption) -> Self {
        Self {
            ifname: opt.ifname,
            buf_size: opt.buf_size,
            timeouts: Timeouts {
                state_transition: Duration::from_secs(10),
                pdu: Duration::from_millis(100),
                wait_loop_delay: Duration::ZERO,
                ..Default::default()
            },
            main_device_config: MainDeviceConfig {
                dc_static_sync_iterations: 10000,
                retry_behaviour: RetryBehaviour::None,
            },
            dc_configuration: DcConfiguration {
                start_delay: Duration::from_millis(100),
                sync0_period: opt.sync0_period,
                sync0_shift: Duration::ZERO,
            },
            state_check_period: opt.state_check_period,
            sync_tolerance: opt.sync_tolerance,
            sync_timeout: opt.sync_timeout,
            tx_rx_thread_builder: opt.tx_rx_thread_builder,
            main_thread_builder: opt.main_thread_builder,
            main_thread_core: opt.main_thread_core,
            tx_rx_thread_core: opt.tx_rx_thread_core,
        }
    }
}
