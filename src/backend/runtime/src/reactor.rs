use std::{
    collections::BTreeMap,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Mutex, OnceLock},
    task::{Context, Poll, Waker},
};

use crate::{
    adapters::storage::{ReaderOf, Storage},
    event::{Event, SignalId},
};

pub struct Reactor {
    blocking_on_read: Mutex<BTreeMap<SignalId, Vec<Waker>>>,
}

impl Reactor {
    pub fn get() -> &'static Reactor {
        static REACTOR: OnceLock<Reactor> = OnceLock::new();

        REACTOR.get_or_init(|| Reactor {
            blocking_on_read: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn react(&self, event: Event) {
        match event {
            Event::SignalReadable(signal_id) => {
                let mut lock = self.blocking_on_read.lock().unwrap();
                if let Some(wakers) = lock.remove(&signal_id) {
                    for waker in wakers {
                        waker.wake();
                    }
                }
            }
        }
    }
}

pub struct FutureSignalValue<S: Storage, T> {
    storage: S,
    addr: S::Addr<T>,
    _pd: PhantomData<T>,
}

impl<S: Storage + ReaderOf<T>, T> Future for FutureSignalValue<S, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.storage.get(&self.addr) {
            return Poll::Ready(value);
        }
        Reactor::get()
            .blocking_on_read
            .lock()
            .unwrap()
            .entry(SignalId(0))
            .or_default()
            .push(cx.waker().clone());
        match self.storage.get(&self.addr) {
            Some(value) => Poll::Ready(value),
            None => Poll::Pending,
        }
    }
}
