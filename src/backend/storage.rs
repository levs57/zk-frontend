pub trait Storage {
    type RawAddr;
}

pub trait AllocatorOf<T>: Storage {
    fn allocate(&mut self) -> <Self as Storage>::RawAddr;
}

pub trait WriterOf<T>: Storage {
    fn put(&mut self, addr: &Self::RawAddr, val: T);
}

pub trait ReaderOf<T>: Storage {
    fn get(&self, addr: &Self::RawAddr) -> &T;
}
