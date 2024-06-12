use std::{
    ffi::{c_void, CString},
    ptr::addr_of_mut,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use async_channel::{bounded, Receiver, SendError, Sender};
use thread_priority::ThreadPriority;
use time::ext::NumericalDuration;

pub use crate::local::builder::SOEMBuilder;

use autd3_driver::{
    error::AUTDInternalError,
    ethercat::{SyncMode, EC_CYCLE_TIME_BASE_NANO_SEC},
    firmware::cpu::{RxMessage, TxDatagram},
    link::Link,
};

use super::{
    error::SOEMError,
    error_handler::{EcatErrorHandler, ErrHandler},
    ethernet_adapters::EthernetAdapters,
    iomap::IOMap,
    osal_timer::Timer,
    sleep::SoemCallback,
    sleep::{BusyWait, Sleep, StdSleep},
    soem_bindings::*,
    state::EcStatus,
    TimerStrategy,
};

pub struct SOEM {
    timeout: Duration,
    sender: Sender<TxDatagram>,
    is_open: Arc<AtomicBool>,
    ec_sync0_cycle: Duration,
    io_map: Arc<Mutex<IOMap>>,
    init_guard: Option<SOEMInitGuard>,
    config_dc_guard: Option<SOEMDCConfigGuard>,
    op_state_guard: Option<OpStateGuard>,
    ecat_th_guard: Option<SOEMECatThreadGuard>,
    ecat_check_th_guard: Option<SOEMEcatCheckThreadGuard>,
}

impl SOEM {
    pub const fn builder() -> SOEMBuilder {
        SOEMBuilder::new()
    }

    pub fn num_devices() -> usize {
        unsafe { ec_slavecount as usize }
    }

    pub async fn clear_iomap(
        &mut self,
    ) -> Result<(), std::sync::PoisonError<std::sync::MutexGuard<'_, IOMap>>> {
        while !self.sender.is_empty() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        self.io_map.lock()?.clear();
        Ok(())
    }
}

unsafe extern "C" fn dc_config(context: *mut ecx_contextt, slave: u16) -> i32 {
    let cyc_time = ((*context).userdata as *mut Duration).as_ref().unwrap();
    ec_dcsync0(slave, 1, cyc_time.as_nanos() as _, 0);
    0
}

impl SOEM {
    pub(crate) async fn open(
        builder: SOEMBuilder,
        geometry: &autd3_driver::geometry::Geometry,
    ) -> Result<Self, AUTDInternalError> {
        let SOEMBuilder {
            buf_size,
            timer_strategy,
            sync_mode,
            ifname,
            state_check_interval,
            timeout,
            sync0_cycle,
            send_cycle,
            thread_priority,
            #[cfg(target_os = "windows")]
            process_priority,
            mut err_handler,
        } = builder;

        if send_cycle == 0 {
            return Err(SOEMError::InvalidSendCycleTime.into());
        }
        if sync0_cycle == 0 {
            return Err(SOEMError::InvalidSync0CycleTime.into());
        }

        let ec_sync0_cycle = Duration::from_nanos(sync0_cycle * EC_CYCLE_TIME_BASE_NANO_SEC);
        let ec_send_cycle = Duration::from_nanos(send_cycle * EC_CYCLE_TIME_BASE_NANO_SEC);
        let ifname = if ifname.is_empty() {
            Self::lookup_autd()?
        } else {
            ifname.clone()
        };

        let init_guard = SOEMInitGuard::new(ifname)?;

        let wc = unsafe { ec_config_init(0) };
        if wc <= 0 || (geometry.num_devices() != 0 && wc as usize != geometry.num_devices()) {
            return Err(SOEMError::SlaveNotFound(wc as _, geometry.len() as _).into());
        }
        (1..=wc).try_for_each(|i| {
            if Self::is_autd3(i) {
                Ok(())
            } else {
                Err(SOEMError::NoDeviceFound)
            }
        })?;
        let num_devices = wc as _;

        let (tx_sender, tx_receiver) = bounded(buf_size);
        let is_open = Arc::new(AtomicBool::new(true));
        let io_map = Arc::new(Mutex::new(IOMap::new(num_devices)));
        let mut result = Self {
            timeout,
            sender: tx_sender,
            is_open,
            ec_sync0_cycle,
            io_map,
            init_guard: Some(init_guard),
            config_dc_guard: Some(SOEMDCConfigGuard::new(sync_mode)),
            op_state_guard: None,
            ecat_th_guard: None,
            ecat_check_th_guard: None,
        };

        result
            .config_dc_guard
            .as_ref()
            .unwrap()
            .configure_dc_dc(ec_sync0_cycle);

        unsafe {
            ec_config_map(result.io_map.lock().unwrap().data() as *mut c_void);
        }

        result.op_state_guard = Some(OpStateGuard::new());
        OpStateGuard::to_safe_op(num_devices)?;
        OpStateGuard::to_op();

        let wkc = Arc::new(AtomicI32::new(0));
        result.ecat_th_guard = Some(SOEMECatThreadGuard::new(
            result.is_open.clone(),
            wkc.clone(),
            result.io_map.clone(),
            tx_receiver,
            timer_strategy,
            thread_priority,
            #[cfg(target_os = "windows")]
            process_priority,
            ec_send_cycle,
        )?);

        if !OpStateGuard::is_op_state() {
            return Err(SOEMError::NotResponding(EcStatus::new(num_devices)).into());
        }

        result
            .config_dc_guard
            .as_ref()
            .unwrap()
            .configure_dc_freerun(ec_sync0_cycle);

        result.ecat_check_th_guard = Some(SOEMEcatCheckThreadGuard::new(
            result.is_open.clone(),
            err_handler.take(),
            wkc.clone(),
            state_check_interval,
        ));

        Ok(result)
    }

    fn is_autd3(i: i32) -> bool {
        unsafe {
            String::from_utf8(
                ec_slave[i as usize]
                    .name
                    .into_iter()
                    .take_while(|&c| c != 0)
                    .map(|c| c as u8)
                    .collect(),
            )
            .map(|name| name == "AUTD")
            .unwrap_or(false)
        }
    }

    fn lookup_autd() -> Result<String, SOEMError> {
        let adapters: EthernetAdapters = Default::default();

        adapters
            .into_iter()
            .find(|adapter| unsafe {
                let ifname = match std::ffi::CString::new(adapter.name().to_owned()) {
                    Ok(ifname) => ifname,
                    Err(_) => return false,
                };
                if ec_init(ifname.as_ptr()) <= 0 {
                    ec_close();
                    return false;
                }
                let wc = ec_config_init(0);
                if wc <= 0 {
                    ec_close();
                    return false;
                }
                let found = (1..=wc).all(Self::is_autd3);
                ec_close();
                found
            })
            .map_or_else(
                || Err(SOEMError::NoDeviceFound),
                |adapter| Ok(adapter.name().to_owned()),
            )
    }
}

#[cfg_attr(feature = "async-trait", autd3_driver::async_trait)]
impl Link for SOEM {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        if !self.is_open() {
            return Ok(());
        }
        self.is_open.store(false, Ordering::Release);

        while !self.sender.is_empty() {
            tokio::time::sleep(self.ec_sync0_cycle).await;
        }

        let _ = self.ecat_th_guard.take();
        let _ = self.ecat_check_th_guard.take();
        let _ = self.config_dc_guard.take();
        let _ = self.op_state_guard.take();
        let _ = self.init_guard.take();

        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        if !self.is_open() {
            return Err(AUTDInternalError::LinkClosed);
        }

        match self.sender.send(tx.clone()).await {
            Err(SendError(..)) => Err(AUTDInternalError::LinkClosed),
            _ => Ok(true),
        }
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        if !self.is_open() {
            return Err(AUTDInternalError::LinkClosed);
        }
        match self.io_map.lock() {
            Ok(io_map) => unsafe {
                std::ptr::copy_nonoverlapping(io_map.input(), rx.as_mut_ptr(), rx.len());
            },
            Err(_) => return Err(AUTDInternalError::LinkClosed),
        }
        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Acquire)
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Drop for SOEM {
    fn drop(&mut self) {
        self.is_open.store(false, Ordering::Release);
        let _ = self.ecat_th_guard.take();
        let _ = self.ecat_check_th_guard.take();
        let _ = self.config_dc_guard.take();
        let _ = self.op_state_guard.take();
        let _ = self.init_guard.take();
    }
}

struct SOEMInitGuard;

impl SOEMInitGuard {
    fn new(ifname: String) -> Result<Self, SOEMError> {
        let ifname_c = CString::new(ifname.as_str())
            .map_err(|_| SOEMError::InvalidInterfaceName(ifname.clone()))?;
        if unsafe { ec_init(ifname_c.as_ptr()) <= 0 } {
            return Err(SOEMError::NoSocketConnection(ifname));
        }
        Ok(Self)
    }
}

impl Drop for SOEMInitGuard {
    fn drop(&mut self) {
        unsafe {
            ec_close();
        }
    }
}

struct SOEMDCConfigGuard {
    sync_mode: SyncMode,
}

impl SOEMDCConfigGuard {
    fn new(sync_mode: SyncMode) -> Self {
        unsafe {
            ecx_context.userdata = std::ptr::null_mut();
        }
        Self { sync_mode }
    }

    fn configure_dc_dc(&self, ec_sync0_cycle: Duration) {
        unsafe {
            if self.sync_mode == SyncMode::DC {
                ecx_context.userdata = Box::into_raw(Box::new(ec_sync0_cycle)) as *mut c_void;
                (1..=ec_slavecount as usize).for_each(|i| {
                    ec_slave[i].PO2SOconfigx = Some(dc_config);
                });
            }
            ec_configdc();
        }
    }

    fn configure_dc_freerun(&self, ec_sync0_cycle: Duration) {
        unsafe {
            if self.sync_mode == SyncMode::FreeRun {
                ecx_context.userdata = Box::into_raw(Box::new(ec_sync0_cycle)) as *mut c_void;
                (1..=ec_slavecount as u16).for_each(|i| {
                    dc_config(addr_of_mut!(ecx_context), i);
                });
            }
        }
    }
}

impl Drop for SOEMDCConfigGuard {
    fn drop(&mut self) {
        unsafe {
            if ecx_context.userdata.is_null() {
                return;
            }
            let cyc_time = Box::from_raw(ecx_context.userdata as *mut Duration);
            ecx_context.userdata = std::ptr::null_mut();
            (1..=ec_slavecount as u16).for_each(|i| {
                ec_dcsync0(i, 0, cyc_time.as_nanos() as _, 0);
            });
        }
    }
}

struct OpStateGuard;

impl OpStateGuard {
    fn new() -> Self {
        Self
    }

    fn to_safe_op(num_devices: usize) -> Result<(), SOEMError> {
        unsafe {
            ec_statecheck(0, ec_state_EC_STATE_SAFE_OP as u16, EC_TIMEOUTSTATE as i32);
            if ec_slave[0].state != ec_state_EC_STATE_SAFE_OP as u16 {
                return Err(SOEMError::NotReachedSafeOp(ec_slave[0].state));
            }
            ec_readstate();
            if ec_slave[0].state != ec_state_EC_STATE_SAFE_OP as u16 {
                return Err(SOEMError::NotResponding(EcStatus::new(num_devices)));
            }
        }

        Ok(())
    }

    fn to_op() {
        unsafe {
            ec_slave[0].state = ec_state_EC_STATE_OPERATIONAL as u16;
            ec_writestate(0);
        }
    }

    fn is_op_state() -> bool {
        unsafe {
            ec_statecheck(
                0,
                ec_state_EC_STATE_OPERATIONAL as u16,
                5 * EC_TIMEOUTSTATE as i32,
            );
            ec_slave[0].state == ec_state_EC_STATE_OPERATIONAL as u16
        }
    }
}

impl Drop for OpStateGuard {
    fn drop(&mut self) {
        unsafe {
            ec_slave[0].state = ec_state_EC_STATE_INIT as u16;
            ec_writestate(0);
        }
    }
}

struct SOEMECatThreadGuard {
    ecatth_handle: Option<JoinHandle<Result<(), SOEMError>>>,
    timer_handle: Option<Box<Timer<SoemCallback>>>,
}

impl SOEMECatThreadGuard {
    #[cfg_attr(target_os = "windows", allow(clippy::too_many_arguments))]
    fn new(
        is_open: Arc<AtomicBool>,
        wkc: Arc<AtomicI32>,
        io_map: Arc<Mutex<IOMap>>,
        tx_receiver: Receiver<TxDatagram>,
        timer_strategy: TimerStrategy,
        thread_priority: ThreadPriority,
        #[cfg(target_os = "windows")] process_priority: super::ProcessPriority,
        ec_send_cycle: Duration,
    ) -> Result<Self, AUTDInternalError> {
        let (ecatth_handle, timer_handle) = match timer_strategy {
            TimerStrategy::Sleep => (
                {
                    Some(std::thread::spawn(move || {
                        Self::ecat_run::<StdSleep>(
                            is_open,
                            io_map,
                            wkc,
                            tx_receiver,
                            ec_send_cycle,
                            thread_priority,
                            #[cfg(target_os = "windows")]
                            process_priority,
                        )
                    }))
                },
                None,
            ),
            TimerStrategy::BusyWait => (
                {
                    Some(std::thread::spawn(move || {
                        Self::ecat_run::<BusyWait>(
                            is_open,
                            io_map,
                            wkc,
                            tx_receiver,
                            ec_send_cycle,
                            thread_priority,
                            #[cfg(target_os = "windows")]
                            process_priority,
                        )
                    }))
                },
                None,
            ),
            TimerStrategy::NativeTimer => (
                None,
                Some(Timer::start(
                    SoemCallback {
                        wkc,
                        receiver: tx_receiver,
                        io_map,
                    },
                    ec_send_cycle,
                )?),
            ),
        };

        Ok(Self {
            ecatth_handle,
            timer_handle,
        })
    }

    fn ecat_run<S: Sleep>(
        is_open: Arc<AtomicBool>,
        io_map: Arc<Mutex<IOMap>>,
        wkc: Arc<AtomicI32>,
        receiver: Receiver<TxDatagram>,
        cycle: Duration,
        thread_priority: ThreadPriority,
        #[cfg(target_os = "windows")] process_priority: super::ProcessPriority,
    ) -> Result<(), SOEMError> {
        unsafe {
            let mut ts = {
                let tp = time::OffsetDateTime::now_utc();
                let tp_unix_ns = tp.unix_timestamp_nanos();
                let cycle_ns = cycle.as_nanos() as i128;
                let ts_unix_ns = (tp_unix_ns / cycle_ns + 1) * cycle_ns;
                time::OffsetDateTime::from_unix_timestamp_nanos(ts_unix_ns).unwrap()
            };

            #[cfg(target_os = "windows")]
            let old_priority = {
                let old_priority = windows::Win32::System::Threading::GetPriorityClass(
                    windows::Win32::System::Threading::GetCurrentProcess(),
                );
                windows::Win32::System::Threading::SetPriorityClass(
                    windows::Win32::System::Threading::GetCurrentProcess(),
                    process_priority.into(),
                )?;
                old_priority
            };

            thread_priority.set_for_current()?;

            let mut toff = time::Duration::ZERO;
            let mut integral = 0;
            ec_send_processdata();
            while is_open.load(Ordering::Acquire) {
                ts += cycle;
                ts += toff;

                S::sleep(ts - time::OffsetDateTime::now_utc());

                wkc.store(
                    ec_receive_processdata(EC_TIMEOUTRET as i32),
                    Ordering::Relaxed,
                );

                toff = Self::ec_sync(ec_DCtime, cycle.as_nanos() as _, &mut integral);

                if let Ok(tx) = receiver.try_recv() {
                    match io_map.lock() {
                        Ok(mut io_map) => io_map.copy_from(&tx),
                        Err(_) => {
                            is_open.store(false, Ordering::Release);
                            break;
                        }
                    }
                }
                ec_send_processdata();
            }

            #[cfg(target_os = "windows")]
            {
                windows::Win32::System::Threading::SetPriorityClass(
                    windows::Win32::System::Threading::GetCurrentProcess(),
                    windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(old_priority),
                )?;
            }
        }
        Ok(())
    }

    fn ec_sync(reftime: i64, cycletime: i64, integral: &mut i64) -> time::Duration {
        let mut delta = (reftime - 50000) % cycletime;
        if delta > (cycletime / 2) {
            delta -= cycletime;
        }
        if delta > 0 {
            *integral += 1;
        }
        if delta < 0 {
            *integral -= 1;
        }
        (-(delta / 100) - (*integral / 20)).nanoseconds()
    }
}

impl Drop for SOEMECatThreadGuard {
    fn drop(&mut self) {
        if let Some(timer) = self.ecatth_handle.take() {
            let _ = timer.join();
        }
        if let Some(timer) = self.timer_handle.take() {
            let _ = timer.close();
        }
    }
}

struct SOEMEcatCheckThreadGuard {
    ecat_check_th: Option<JoinHandle<()>>,
}

impl SOEMEcatCheckThreadGuard {
    fn new(
        is_open: Arc<AtomicBool>,
        err_handler: Option<ErrHandler>,
        wkc: Arc<AtomicI32>,
        state_check_interval: Duration,
    ) -> Self {
        let expected_wkc = unsafe { (ec_group[0].outputsWKC * 2 + ec_group[0].inputsWKC) as i32 };
        Self {
            ecat_check_th: Some(std::thread::spawn(move || {
                let err_handler = EcatErrorHandler { err_handler };
                err_handler.run(is_open, wkc, expected_wkc, state_check_interval)
            })),
        }
    }
}

impl Drop for SOEMEcatCheckThreadGuard {
    fn drop(&mut self) {
        if let Some(th) = self.ecat_check_th.take() {
            let _ = th.join();
        }
    }
}
