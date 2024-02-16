use std::{
    collections::VecDeque,
    pin::Pin,
    sync::{Mutex, OnceLock},
    task::Poll,
};

use futures::{stream::Stream, task::AtomicWaker, StreamExt};

use crate::reactor::Reactor;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Event {
    SignalReadable(SignalId),
}

static EVENT_QUEUE: OnceLock<Mutex<VecDeque<Event>>> = OnceLock::new();
static EVENT_WAKER: AtomicWaker = AtomicWaker::new();

pub struct EventStream(()); // private field prevents construction from outside the module

impl EventStream {
    pub fn new() -> Self {
        let _ = EVENT_QUEUE.get_or_init(|| Mutex::default());
        Self(())
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
        if let Some(event) = event_queue.lock().unwrap().pop_back() {
            return Poll::Ready(Some(event));
        }
        EVENT_WAKER.register(&cx.waker());
        match event_queue.lock().unwrap().pop_back() {
            Some(event) => {
                EVENT_WAKER.take();
                Poll::Ready(Some(event))
            }
            None => Poll::Pending,
        }
    }
}

pub fn emit(event: Event) {
    println!("emitting {event:?}");
    let event_queue = EVENT_QUEUE
        .get()
        .expect("event queue should be initialized before emitting");
    event_queue.lock().unwrap().push_back(event);
    EVENT_WAKER.wake();
}

pub async fn process_events() {
    println!("process_events");
    let mut event_stream = EventStream::new();
    let reactor = Reactor::get();

    while let Some(event) = event_stream.next().await {
        println!("process_event reacting on {event:?}");
        reactor.react(event);
    }
}
