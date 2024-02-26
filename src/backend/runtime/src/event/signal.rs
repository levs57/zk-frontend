use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use crate::{
    reactor::Reactor,
    storage::{ReaderOf, Storage},
};

/// A tag identifying the signal that would become readable
/// via an `Event::SignalReadable` event.
///
/// Current implementation also assumes this is an address
/// used for all awaitable value kinds in storage
/// (i.e, `Storage::Addr<T> = SignalId`).
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(pub usize);

/// A future waiting on a signal to obtain value in storage.
///
/// Awaiting it will pause currently running task until an
/// `Event::SignalReadable` event is received for the given address.
/// Returns the signal's value.
pub struct Signal<S: Storage, T> {
    storage: Arc<Mutex<S>>,
    addr: S::Addr<T>,
}

impl<S: Storage, T> Signal<S, T> {
    pub fn new(storage: Arc<Mutex<S>>, addr: S::Addr<T>) -> Self {
        Self { storage, addr }
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
