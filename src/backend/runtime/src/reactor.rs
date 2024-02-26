use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
    task::Waker,
};

use crate::event::{signal::SignalId, Event};

/// Asyncronous event handler.
///
/// This is a low-level abstraction. Rather than using it directly,
/// consider using the futures from the `event` module.
///  
/// Handles any side effects arising inside a running coroutine/future.
/// Side effects include signals becoming readable, round bumps
/// and circuit IO. "Handling" here means notifying any waiting coroutines
/// that an event has occured, possibly resuming their execution.
///
/// The way to obtain a `Reactor` instance is through `Reactor::get()`
/// method. This way you don't need to pass the reactor through all the
/// async functions on the stack in order to use it in the leaf futures.
///
/// Since a reactor is an immutable singleton (stored inside a scoped static),
/// all of its fields have to be wrapped in interior-mutable
/// thread-safe structures.
pub struct Reactor {
    blocking_on_read: Mutex<BTreeMap<SignalId, Vec<Waker>>>,
}

impl Reactor {
    /// Get the reactor instance. If it's not initialized, also initialize it.
    ///
    /// It is the only way to obtain a reactor instance.
    pub fn get() -> &'static Reactor {
        static REACTOR: OnceLock<Reactor> = OnceLock::new();

        REACTOR.get_or_init(|| Reactor {
            blocking_on_read: Mutex::new(BTreeMap::new()),
        })
    }

    /// Arrange for a wakeup when a signal becomes readable.
    pub fn register_reader(&self, signal_id: SignalId, waker: Waker) {
        self.blocking_on_read
            .lock()
            .unwrap()
            .entry(signal_id)
            .or_default()
            .push(waker);
    }

    /// The event handler.
    ///
    /// Will be called for every event emitted.
    /// May be called more than once per event.
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
