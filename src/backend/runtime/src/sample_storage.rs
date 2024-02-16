use crate::{
    adapters::storage::{AllocatorOf, ReaderOf, Storage, WriterOf},
    event::SignalId,
};

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
