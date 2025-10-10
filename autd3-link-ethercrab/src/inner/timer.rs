use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    sync::{Arc, Condvar, Mutex, OnceLock},
    task::Waker,
    time::{Duration, Instant},
};

struct TimerEntry {
    deadline: Instant,
    waker: Waker,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

impl Eq for TimerEntry {}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.deadline.cmp(&self.deadline)
    }
}

struct TimerQueue {
    heap: Mutex<BinaryHeap<TimerEntry>>,
    condvar: Condvar,
}

impl TimerQueue {
    fn new() -> Arc<Self> {
        let queue = Arc::new(Self {
            heap: Mutex::new(BinaryHeap::new()),
            condvar: Condvar::new(),
        });

        let queue_clone = queue.clone();
        std::thread::spawn(move || {
            queue_clone.run();
        });

        queue
    }

    fn insert(&self, deadline: Instant, waker: Waker) {
        let mut heap = self.heap.lock().unwrap();
        heap.push(TimerEntry { deadline, waker });
        self.condvar.notify_one();
    }

    fn run(&self) {
        loop {
            let mut heap = self.heap.lock().unwrap();

            while let Some(entry) = heap.peek() {
                let now = Instant::now();
                if now >= entry.deadline {
                    let entry = heap.pop().unwrap();
                    drop(heap);
                    entry.waker.wake();
                    heap = self.heap.lock().unwrap();
                } else {
                    let sleep_duration = entry.deadline.saturating_duration_since(now);
                    let (new_heap, timeout_result) =
                        self.condvar.wait_timeout(heap, sleep_duration).unwrap();
                    heap = new_heap;
                    if !timeout_result.timed_out() {
                        continue;
                    }
                    break;
                }
            }

            if heap.is_empty() {
                drop(self.condvar.wait(heap).unwrap());
            }
        }
    }
}

static TIMER_QUEUE: OnceLock<Arc<TimerQueue>> = OnceLock::new();

fn get_timer_queue() -> &'static Arc<TimerQueue> {
    TIMER_QUEUE.get_or_init(TimerQueue::new)
}

pub(crate) struct Timer {
    deadline: Instant,
    registered: bool,
}

impl Timer {
    pub fn after(dur: Duration) -> Self {
        Self::until(Instant::now() + dur)
    }

    pub fn until(deadline: Instant) -> Self {
        Self {
            deadline,
            registered: false,
        }
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            std::task::Poll::Ready(())
        } else {
            if !self.registered {
                get_timer_queue().insert(self.deadline, cx.waker().clone());
                self.registered = true;
            }
            std::task::Poll::Pending
        }
    }
}
