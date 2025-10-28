use super::waker::Waker;

use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll};

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
