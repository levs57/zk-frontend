use std::{
    collections::BTreeMap,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex, OnceLock},
    task::{Context, Poll, Waker},
};

use crate::{
    event::{Event, SignalId},
    storage::{ReaderOf, Storage},
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

    pub fn register_reader(&self, signal_id: SignalId, waker: Waker) {
        self.blocking_on_read
            .lock()
            .unwrap()
            .entry(signal_id)
            .or_default()
            .push(waker);
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

pub struct Signal<S: Storage, T> {
    storage: Arc<Mutex<S>>,
    addr: S::Addr<T>,
    _pd: PhantomData<T>,
}

impl<S: Storage, T> Signal<S, T> {
    pub fn new(storage: Arc<Mutex<S>>, addr: S::Addr<T>) -> Self {
        Self {
            storage,
            addr,
            _pd: PhantomData,
        }
    }
}

impl<S: Storage<Addr<T> = SignalId> + ReaderOf<T>, T> Future for Signal<S, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(value) = self.storage.get(&self.addr) {
            return Poll::Ready(value);
        }
        Reactor::get().register_reader(self.addr, cx.waker().clone());
        match self.storage.get(&self.addr) {
            Some(value) => Poll::Ready(value),
            None => Poll::Pending,
        }
    }
}
