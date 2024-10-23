use std::time::Duration;

pub(crate) trait Sleep {
    fn sleep(duration: Duration);
}

pub(crate) struct StdSleep {}

impl Sleep for StdSleep {
    fn sleep(duration: Duration) {
        #[cfg(target_os = "windows")]
        unsafe {
            windows::Win32::Media::timeBeginPeriod(1);
        }
        std::thread::sleep(duration);
        #[cfg(target_os = "windows")]
        unsafe {
            windows::Win32::Media::timeEndPeriod(1);
        }
    }
}

pub(crate) struct SpinSleep {}

impl Sleep for SpinSleep {
    fn sleep(duration: Duration) {
        spin_sleep::sleep(duration);
    }
}

pub(crate) struct SpinWait {}

impl Sleep for SpinWait {
    fn sleep(duration: Duration) {
        let expired = std::time::Instant::now() + duration;
        while std::time::Instant::now() < expired {
            std::hint::spin_loop();
        }
    }
}
