use std::{
    ffi::c_void,
    ptr::addr_of_mut,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use async_channel::{bounded, Receiver, SendError, Sender};
use time::ext::NumericalDuration;

use autd3_driver::{
    cpu::{RxMessage, TxDatagram, EC_CYCLE_TIME_BASE_NANO_SEC},
    error::AUTDInternalError,
    link::Link,
    osal_timer::Timer,
    sync_mode::SyncMode,
    timer_strategy::TimerStrategy,
};

pub use crate::local::builder::SOEMBuilder;
use crate::{
    local::{error::SOEMError, iomap::IOMap, sleep::SoemCallback, soem_bindings::*},
    EthernetAdapters,
};

use super::{
    error_handler::EcatErrorHandler,
    sleep::{BusyWait, Sleep, StdSleep},
    state::EcStatus,
};

/// Link using [SOEM](https://github.com/OpenEtherCATsociety/SOEM)
pub struct SOEM {
    ecatth_handle: Option<JoinHandle<Result<(), SOEMError>>>,
    timer_handle: Option<Box<Timer<SoemCallback>>>,
    ecat_check_th: Option<JoinHandle<()>>,
    timeout: std::time::Duration,
    sender: Sender<TxDatagram>,
    is_open: Arc<AtomicBool>,
    ec_sync0_cycle: std::time::Duration,
    io_map: Arc<Mutex<IOMap>>,
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
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        self.io_map.lock()?.clear();
        Ok(())
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
            let found = (1..=wc).all(|i| {
                match String::from_utf8(
                    ec_slave[i as usize]
                        .name
                        .iter()
                        .take_while(|&&c| c != 0)
                        .map(|&c| c as u8)
                        .collect(),
                ) {
                    Ok(name) => name == "AUTD",
                    Err(_) => false,
                }
            });
            ec_close();
            found
        })
        .map_or_else(
            || Err(SOEMError::NoDeviceFound),
            |adapter| Ok(adapter.name().to_owned()),
        )
}

unsafe extern "C" fn dc_config(context: *mut ecx_contextt, slave: u16) -> i32 {
    let cyc_time = *((*context).userdata as *mut std::time::Duration);
    ec_dcsync0(slave, 1, cyc_time.as_nanos() as _, 0);
    0
}

fn ecat_run<S: Sleep>(
    is_open: Arc<AtomicBool>,
    io_map: Arc<Mutex<IOMap>>,
    wkc: Arc<AtomicI32>,
    receiver: Receiver<TxDatagram>,
    cycle: std::time::Duration,
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
        let priority = {
            let priority = windows::Win32::System::Threading::GetPriorityClass(
                windows::Win32::System::Threading::GetCurrentProcess(),
            );
            windows::Win32::System::Threading::SetPriorityClass(
                windows::Win32::System::Threading::GetCurrentProcess(),
                windows::Win32::System::Threading::REALTIME_PRIORITY_CLASS,
            )?;
            windows::Win32::System::Threading::SetThreadPriority(
                windows::Win32::System::Threading::GetCurrentThread(),
                windows::Win32::System::Threading::THREAD_PRIORITY_TIME_CRITICAL,
            )?;
            windows::Win32::Media::timeBeginPeriod(1);
            priority
        };

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

            toff = ec_sync(ec_DCtime, cycle.as_nanos() as _, &mut integral);

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
            windows::Win32::Media::timeEndPeriod(1);
            windows::Win32::System::Threading::SetPriorityClass(
                windows::Win32::System::Threading::GetCurrentProcess(),
                windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(priority),
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
            mut err_handler,
        } = builder;

        unsafe {
            let ec_sync0_cycle =
                std::time::Duration::from_nanos(sync0_cycle * EC_CYCLE_TIME_BASE_NANO_SEC);
            let ec_send_cycle =
                std::time::Duration::from_nanos(send_cycle * EC_CYCLE_TIME_BASE_NANO_SEC);
            let num_devices = {
                if send_cycle == 0 {
                    return Err(SOEMError::InvalidSendCycleTime.into());
                }
                if sync0_cycle == 0 {
                    return Err(SOEMError::InvalidSync0CycleTime.into());
                }

                let ifname = if ifname.is_empty() {
                    lookup_autd()?
                } else {
                    ifname.clone()
                };
                let ifname_c = match std::ffi::CString::new(ifname.clone()) {
                    Ok(ifname) => ifname,
                    Err(_) => return Err(SOEMError::InvalidInterfaceName(ifname).into()),
                };

                if ec_init(ifname_c.as_ptr()) <= 0 {
                    return Err(SOEMError::NoSocketConnection(ifname).into());
                }

                let wc = ec_config_init(0);
                if wc <= 0 {
                    return Err(SOEMError::SlaveNotFound(0, geometry.len() as _).into());
                }

                if let Err(e) = (1..=wc as usize)
                    .map(|i| {
                        if let Ok(name) = String::from_utf8(
                            ec_slave[i]
                                .name
                                .iter()
                                .take_while(|&&c| c != 0)
                                .map(|&c| c as u8)
                                .collect(),
                        ) {
                            if name.is_empty() {
                                Err(SOEMError::NoDeviceFound)
                            } else if name == "AUTD" {
                                Ok(())
                            } else {
                                Err(SOEMError::NotAUTD3Device(name))
                            }
                        } else {
                            Err(SOEMError::NoDeviceFound)
                        }
                    })
                    .collect::<Result<Vec<()>, SOEMError>>()
                {
                    return Err(e.into());
                }

                ecx_context.userdata = Box::into_raw(Box::new(ec_sync0_cycle)) as *mut c_void;
                match sync_mode {
                    SyncMode::DC => {
                        (1..=ec_slavecount as usize).for_each(|i| {
                            ec_slave[i].PO2SOconfigx = Some(dc_config);
                        });
                    }
                    SyncMode::FreeRun => (),
                }

                ec_configdc();

                if geometry.num_devices() != 0 && wc as usize != geometry.num_devices() {
                    return Err(
                        SOEMError::SlaveNotFound(wc as _, geometry.num_devices() as _).into(),
                    );
                }
                wc as _
            };

            let io_map = IOMap::new(num_devices);
            ec_config_map(io_map.data() as *mut c_void);
            let io_map = Arc::new(Mutex::new(io_map));

            ec_statecheck(0, ec_state_EC_STATE_SAFE_OP as u16, EC_TIMEOUTSTATE as i32);
            if ec_slave[0].state != ec_state_EC_STATE_SAFE_OP as u16 {
                return Err(SOEMError::NotReachedSafeOp(ec_slave[0].state).into());
            }
            ec_readstate();
            if ec_slave[0].state != ec_state_EC_STATE_SAFE_OP as u16 {
                return Err(SOEMError::NotResponding(EcStatus::new(num_devices)).into());
            }

            ec_slave[0].state = ec_state_EC_STATE_OPERATIONAL as u16;
            ec_writestate(0);

            let is_open = Arc::new(AtomicBool::new(true));
            let wkc = Arc::new(AtomicI32::new(0));
            let (tx_sender, tx_receiver) = bounded(buf_size);

            let (mut ecatth_handle, mut timer_handle) = match timer_strategy {
                TimerStrategy::Sleep => (
                    {
                        let is_open = is_open.clone();
                        let io_map = io_map.clone();
                        let wkc = wkc.clone();
                        Some(std::thread::spawn(move || {
                            ecat_run::<StdSleep>(is_open, io_map, wkc, tx_receiver, ec_send_cycle)
                        }))
                    },
                    None,
                ),
                TimerStrategy::BusyWait => (
                    {
                        let is_open = is_open.clone();
                        let io_map = io_map.clone();
                        let wkc = wkc.clone();
                        Some(std::thread::spawn(move || {
                            ecat_run::<BusyWait>(is_open, io_map, wkc, tx_receiver, ec_send_cycle)
                        }))
                    },
                    None,
                ),
                TimerStrategy::NativeTimer => (
                    None,
                    Some(Timer::start(
                        SoemCallback {
                            wkc: wkc.clone(),
                            receiver: tx_receiver,
                            io_map: io_map.clone(),
                        },
                        ec_send_cycle,
                    )?),
                ),
            };

            ec_statecheck(
                0,
                ec_state_EC_STATE_OPERATIONAL as u16,
                5 * EC_TIMEOUTSTATE as i32,
            );
            if ec_slave[0].state != ec_state_EC_STATE_OPERATIONAL as u16 {
                is_open.store(false, Ordering::Release);
                if let Some(timer) = ecatth_handle.take() {
                    let _ = timer.join();
                }
                if let Some(timer) = timer_handle.take() {
                    timer.close()?;
                }

                return Err(SOEMError::NotResponding(EcStatus::new(num_devices)).into());
            }

            match sync_mode {
                SyncMode::DC => (),
                SyncMode::FreeRun => {
                    (1..=ec_slavecount as u16).for_each(|i| {
                        dc_config(addr_of_mut!(ecx_context) as *mut _, i);
                    });
                }
            }

            let ecat_check_th = Some({
                let expected_wkc = (ec_group[0].outputsWKC * 2 + ec_group[0].inputsWKC) as i32;
                let is_open = is_open.clone();
                let err_handler = err_handler.take();
                std::thread::spawn(move || {
                    let err_handler = EcatErrorHandler { err_handler };
                    err_handler.run(is_open, wkc, expected_wkc, state_check_interval)
                })
            });

            Ok(Self {
                ecatth_handle,
                timer_handle,
                ecat_check_th,
                timeout,
                sender: tx_sender,
                is_open,
                ec_sync0_cycle,
                io_map,
            })
        }
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

        if let Some(timer) = self.ecatth_handle.take() {
            let _ = timer.join();
        }
        if let Some(timer) = self.timer_handle.take() {
            timer.close()?;
        }
        if let Some(th) = self.ecat_check_th.take() {
            let _ = th.join();
        }

        unsafe {
            let cyc_time = *(ecx_context.userdata as *mut u32);
            (1..=ec_slavecount as u16).for_each(|i| {
                ec_dcsync0(i, 0, cyc_time, 0);
            });

            ec_slave[0].state = ec_state_EC_STATE_INIT as _;
            ec_writestate(0);

            ec_close();

            let _ = Box::from_raw(ecx_context.userdata as *mut std::time::Duration);
        }

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
