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
    fn get(&self, addr: &Self::Addr<T>) -> &T;
}
