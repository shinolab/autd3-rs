use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};

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

pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);

    let waker_impl = Arc::new(Waker::new());
    let waker = std::task::Waker::from(Arc::clone(&waker_impl));
    let mut context = Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                waker_impl.wait();
            }
        }
    }
}
