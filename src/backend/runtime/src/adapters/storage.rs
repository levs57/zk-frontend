use std::sync::{Arc, Mutex};

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
