use std::{future::{self, Future}, pin::Pin, sync::{Arc, Mutex, RwLock}, task::{Poll, Waker}};

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


pub trait AsyncStorage<V>: Storage
where
    Self: ReaderOf<V> + WriterOf<V>
{
    fn make_future_for(&self, signal: &Self::Addr<V>) -> Pin<Box<dyn Future<Output = V>>>;
    fn register_waker_for(&mut self, signal: &Self::Addr<V>, waker: Waker);
    fn emit(&mut self, signal: &Self::Addr<V>, value: V);
}

// SAMPLE IMPLEMENTATION

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(usize);

pub struct MyStorage {
    data: Vec<Option<usize>>,
    wakers: RwLock<Vec<Vec<Waker>>>,
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
    fn maybe_get(&self, addr: &Self::Addr<usize>) -> Option<&usize> {
        self.data[addr.0].as_ref()
    }
}

impl WriterOf<usize> for MyStorage {
    fn put(&mut self, addr: &Self::Addr<usize>, val: usize) {
        assert!(self.data[addr.0].is_none());
        self.data[addr.0] = Some(val);
    }
}

// pub struct FutureStorageValue<S: Storage, V>(Arc<Mutex<S>>, S::Addr<V>);

// impl<V: Clone> Future for FutureStorageValue<MyStorage, V>
// where
//     MyStorage: AsyncStorage<V>,
//     MyStorage: ReaderOf<V>,
// {
//     type Output = V;

//     fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
//         let storage = self.0.get_mut().unwrap();

//         if let Some(value) = storage.maybe_get(&self.1) {
//             return Poll::Ready(value.clone());
//         }
        
//         storage.register_waker_for(&self.1, cx.waker().clone());
//         match storage.maybe_get(&self.1) {
//             Some(value) => Poll::Ready(value.clone()),
//             None => Poll::Pending,
//         }
//     }
// }

// impl AsyncStorage<usize> for MyStorage {
//     fn make_future_for(&self, signal: &Self::Addr<usize>) -> Pin<Box<(dyn Future<Output = usize> + 'static)>> {
//         match self.maybe_get(signal) {
//             Some(value) => Box::pin(future::ready(*value)) as Pin<Box<(dyn Future<Output = usize> + 'static)>>,
//             None => Box::pin(FutureStorageValue(*signal)) as Pin<Box<(dyn Future<Output = usize> + 'static)>>,
//         }
//     }

//     fn register_waker_for(&mut self, signal: &Self::Addr<usize>, waker: Waker) {
//         let mut wakers = self.wakers.write().unwrap();
//         wakers[signal.0].push(waker);
//     }

//     fn emit(&mut self, signal: &Self::Addr<usize>, value: usize) {
//         self.put(signal, value);

//         let read_lock = self.wakers.read().unwrap();
//         for waker in &read_lock[signal.0] {
//             waker.wake_by_ref();
//         }
//     }
// }
