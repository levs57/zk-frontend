use std::{
    collections::BTreeMap, future::Future, marker::PhantomData, pin::Pin, sync::{Mutex, OnceLock}, task::{Context, Poll, Waker}
};

use crate::adapters::storage::{AsyncStorage, MyStorage, ReaderOf, SignalId};

pub struct Reactor {
    storage: Mutex<MyStorage>,
    tasks: Mutex<BTreeMap<SignalId, Vec<Waker>>>,
}

impl Reactor {
    pub fn get() -> &'static Reactor {
        static REACTOR: OnceLock<Reactor> = OnceLock::new();

        REACTOR.get_or_init(|| Reactor {
            storage: Mutex::new(MyStorage::new()),
            tasks: Mutex::new(BTreeMap::new()),
        })
    }

    pub fn register(&self, signal_id: SignalId, waker: &Waker) {
        {
            let mut storage = self.storage.lock().unwrap();
            AsyncStorage::<usize>::register_interest_in(&mut *storage, signal_id);
        }
        self.tasks
            .lock()
            .unwrap()
            .entry(signal_id)
            .or_insert(Vec::new())
            .push(waker.clone());
    }

    pub fn heartbeat(&self) {
        let storage = self.storage.lock().unwrap();
        let tasks = self.tasks.lock().unwrap();
        for signal_id in &AsyncStorage::<usize>::poll(&*storage) {
            let wakers = tasks.get(signal_id).expect("poll should not return unregistred ids");
            for waker in wakers {
                waker.wake_by_ref();
            }
        }
    }
}

pub struct SignalFuture<T>(pub SignalId, PhantomData<T>);

impl<T> Future for SignalFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let reactor = Reactor::get();
        if let Some(value) = reactor.storage.lock().unwrap().get(&self.0) {
            return Poll::Ready(*value);
        }
        reactor.register(self.0, cx.waker());
        match reactor.storage.lock().unwrap().get(&self.0) {
            Some(value) => Poll::Ready(*value),
            None => Poll::Pending,
        }
    }
}
