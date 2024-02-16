use std::{cell::Cell, collections::BTreeSet};

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
    fn get(&self, addr: &Self::Addr<T>) -> Option<&T>;
}

pub trait AsyncStorage<V>: Storage {
    fn register_interest_in(&mut self, addr: Self::Addr<V>);
    fn poll(&self) -> Vec<Self::Addr<V>>;
}

// SAMPLE IMPLEMENTATION

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(usize);

pub struct MyStorage {
    data: Vec<Option<usize>>,
    interests: BTreeSet<SignalId>,
    ready: Cell<Vec<SignalId>>,
}

impl MyStorage {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            interests: BTreeSet::new(),
            ready: Cell::new(Vec::new()),
        }
    }
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
    fn get(&self, addr: &Self::Addr<usize>) -> Option<&usize> {
        self.data[addr.0].as_ref()
    }
}

impl WriterOf<usize> for MyStorage {
    fn put(&mut self, addr: &Self::Addr<usize>, val: usize) {
        assert!(self.data[addr.0].is_none());
        self.data[addr.0] = Some(val);

        if self.interests.contains(&addr) {
            self.ready.get_mut().push(*addr); // no double writes so no duplicates
        }
    }
}

impl<V> AsyncStorage<V> for MyStorage {
    fn register_interest_in(&mut self, addr: Self::Addr<V>) {
        self.interests.insert(addr);
    }

    fn poll(&self) -> Vec<Self::Addr<V>> {
        // TODO: cycle two buffers instead of allocating new one every time
        self.ready.replace(Vec::new())
    }
}
