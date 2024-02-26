use std::sync::{Arc, Mutex};

use crate::event::signal::SignalId;

pub trait Storage {
    type Addr<T>;
}

pub trait AllocatorOf<T>: Storage {
    fn allocate(&mut self) -> <Self as Storage>::Addr<T>;
}

pub trait WriterOf<T>: Storage {
    fn put(&mut self, addr: &Self::Addr<T>, val: T);
}

pub trait ReaderOf<T>: Storage {
    fn get(&self, addr: &Self::Addr<T>) -> Option<T>;
}

impl<S: Storage> Storage for Arc<Mutex<S>> {
    type Addr<T> = S::Addr<T>;
}

impl<S: AllocatorOf<T>, T> AllocatorOf<T> for Arc<Mutex<S>> {
    fn allocate(&mut self) -> <Self as Storage>::Addr<T> {
        let mut lock = self.as_ref().lock().unwrap();
        lock.allocate()
    }
}

impl<S: ReaderOf<T>, T> ReaderOf<T> for Arc<Mutex<S>> {
    fn get(&self, addr: &Self::Addr<T>) -> Option<T> {
        let lock = self.as_ref().lock().unwrap();
        lock.get(addr)
    }
}

impl<S: WriterOf<T>, T> WriterOf<T> for Arc<Mutex<S>> {
    fn put(&mut self, addr: &Self::Addr<T>, val: T) {
        let mut lock = self.as_ref().lock().unwrap();
        lock.put(addr, val)
    }
}

#[derive(Default)]
pub struct MyStorage {
    data: Vec<Option<usize>>,
}

impl Storage for MyStorage {
    type Addr<T> = SignalId;
}

impl AllocatorOf<usize> for MyStorage {
    fn allocate(&mut self) -> <Self as Storage>::Addr<usize> {
        self.data.push(None);
        SignalId(self.data.len() - 1)
    }
}

impl ReaderOf<usize> for MyStorage {
    fn get(&self, addr: &Self::Addr<usize>) -> Option<usize> {
        self.data[addr.0]
    }
}

impl WriterOf<usize> for MyStorage {
    fn put(&mut self, addr: &Self::Addr<usize>, val: usize) {
        assert!(self.data[addr.0].is_none());
        self.data[addr.0] = Some(val);
    }
}

pub type SharedStorage = Arc<Mutex<MyStorage>>;
