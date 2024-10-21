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

pub(crate) struct SpinSleep {}

impl Sleep for SpinSleep {
    fn sleep(duration: time::Duration) {
        spin_sleep::sleep(std::time::Duration::from_nanos(
            duration.whole_nanoseconds() as _,
        ));
    }
}

pub(crate) struct SpinWait {}

impl Sleep for SpinWait {
    fn sleep(duration: time::Duration) {
        let expired = time::OffsetDateTime::now_utc() + duration;
        while time::OffsetDateTime::now_utc() < expired {
            std::hint::spin_loop();
        }
    }
}
