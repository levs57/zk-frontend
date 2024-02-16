use std::{collections::VecDeque, pin::Pin, sync::OnceLock, task::Poll};

use atomic_waker::AtomicWaker;
use futures::{stream::Stream, StreamExt};

use crate::reactor::Reactor;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(pub usize);

pub enum Event {
    SignalReadable(SignalId),
}

static EVENT_QUEUE: OnceLock<VecDeque<Event>> = OnceLock::new(); // TODO: thread safe queue
static EVENT_WAKER: AtomicWaker = AtomicWaker::new();

pub struct EventStream(()); // private field prevents construction from outside the module

impl EventStream {
    pub fn new() -> EventStream {
        let _ = EVENT_QUEUE.get_or_init(|| VecDeque::new());
        EventStream(())
    }
}

impl Stream for EventStream {
    type Item = Event;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let event_queue = EVENT_QUEUE
            .get()
            .expect("event queue should be initialized before polling");
        // possible race condition:
        // Thread A            Thread B
        // queue check         |
        // |                   |
        // |                   emit(...) --- no task awoken
        // |                   |
        // waker registration  |
        if let Some(event) = event_queue.pop_back() {
            return Poll::Ready(Some(event));
        }
        EVENT_WAKER.register(&cx.waker());
        match event_queue.pop_back() {
            Some(event) => {
                EVENT_WAKER.take();
                Poll::Ready(Some(event))
            }
            None => Poll::Pending,
        }
    }
}

pub fn emit(event: Event) {
    let event_queue = EVENT_QUEUE
        .get_mut()
        .expect("event queue should be initialized before emitting");
    event_queue.push_back(event);
    EVENT_WAKER.wake();
}

pub async fn process_events() {
    let mut event_stream = EventStream::new();
    let reactor = Reactor::get();

    while let Some(event) = event_stream.next().await {
        reactor.react(event);
    }
}
