use std::marker::PhantomData;

pub struct TypedAddr<S: Storage, T> {
    pub addr: S::RawAddr,
    pub _pd: PhantomData<T>,
}

pub trait Storage {
    type RawAddr;

    fn to_raw<T>(ta: &TypedAddr<Self, T>) -> Self::RawAddr where Self: Sized;
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
