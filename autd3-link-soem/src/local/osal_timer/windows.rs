use windows::Win32::{Foundation::*, System::Threading::*};

use autd3_driver::error::AUTDInternalError;

pub struct NativeTimerWrapper {
    h_queue: HANDLE,
    h_timer: HANDLE,
    h_process: HANDLE,
    priority: u32,
}

impl NativeTimerWrapper {
    pub fn new() -> NativeTimerWrapper {
        NativeTimerWrapper {
            h_queue: HANDLE::default(),
            h_timer: HANDLE::default(),
            h_process: HANDLE::default(),
            priority: 0,
        }
    }

    // GRCOV_EXCL_START
    pub fn start<P>(
        &mut self,
        cb: WAITORTIMERCALLBACK,
        period: std::time::Duration,
        lp_param: *mut P,
    ) -> Result<bool, AUTDInternalError> {
        unsafe {
            self.h_process = GetCurrentProcess();
            self.priority = GetPriorityClass(self.h_process);
            let _ = SetPriorityClass(self.h_process, REALTIME_PRIORITY_CLASS);

            let interval = (period.as_nanos() / 1000 / 1000) as u32;

            self.h_queue = CreateTimerQueue()?;
            CreateTimerQueueTimer(
                &mut self.h_timer as *mut _,
                self.h_queue,
                cb,
                Some(lp_param as *const _),
                0,
                interval.max(1),
                WORKER_THREAD_FLAGS(0),
            )?;

            Ok(true)
        }
    }
    // GRCOV_EXCL_STOP

    // GRCOV_EXCL_START
    pub fn close(&mut self) -> Result<(), AUTDInternalError> {
        unsafe {
            if !self.h_timer.is_invalid() {
                DeleteTimerQueueTimer(self.h_queue, self.h_timer, None)?;
                DeleteTimerQueue(self.h_queue)?;

                let _ = SetPriorityClass(
                    self.h_process,
                    windows::Win32::System::Threading::PROCESS_CREATION_FLAGS(self.priority),
                );

                self.h_queue = HANDLE::default();
                self.h_timer = HANDLE::default();
            }
        }
        Ok(())
    }
    // GRCOV_EXCL_STOP
}

impl Drop for NativeTimerWrapper {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
