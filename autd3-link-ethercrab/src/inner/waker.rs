use std::{sync::Arc, task::Wake};

pub struct Waker {
    thread: std::thread::Thread,
}

impl Waker {
    pub fn new() -> Self {
        Self {
            thread: std::thread::current(),
        }
    }

    pub fn wait(&self) {
        std::thread::park();
    }
}

impl Wake for Waker {
    fn wake_by_ref(self: &Arc<Self>) {
        self.thread.unpark();
    }

    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
}
