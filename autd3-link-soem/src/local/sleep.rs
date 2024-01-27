use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc, Mutex,
};

use async_channel::Receiver;
use autd3_driver::{cpu::TxDatagram, osal_timer::TimerCallback};

use crate::local::{
    iomap::IOMap,
    soem_bindings::{ec_receive_processdata, ec_send_processdata, EC_TIMEOUTRET},
};

pub(crate) trait Sleep {
    fn sleep(duration: time::Duration);
}

pub(crate) struct StdSleep {}

impl Sleep for StdSleep {
    fn sleep(duration: time::Duration) {
        if duration > time::Duration::ZERO {
            std::thread::sleep(std::time::Duration::from_nanos(
                duration.whole_nanoseconds() as _,
            ))
        }
    }
}

pub(crate) struct BusyWait {}

impl Sleep for BusyWait {
    fn sleep(duration: time::Duration) {
        let expired = time::OffsetDateTime::now_utc() + duration;
        while time::OffsetDateTime::now_utc() < expired {
            std::hint::spin_loop();
        }
    }
}

pub(crate) struct SoemCallback {
    pub(crate) wkc: Arc<AtomicI32>,
    pub(crate) receiver: Receiver<TxDatagram>,
    pub(crate) io_map: Arc<Mutex<IOMap>>,
}

impl TimerCallback for SoemCallback {
    fn rt_thread(&mut self) {
        unsafe {
            ec_send_processdata();
            self.wkc.store(
                ec_receive_processdata(EC_TIMEOUTRET as i32),
                Ordering::Relaxed,
            );

            if let Ok(tx) = self.receiver.try_recv() {
                if let Ok(mut io_map) = self.io_map.lock() {
                    io_map.copy_from(&tx);
                }
            }
        }
    }
}
