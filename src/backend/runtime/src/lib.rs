pub mod adapters;
pub mod event;
pub mod executor;
pub mod reactor;
pub mod sample_storage;
pub mod task;
pub mod waker;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        adapters::storage::{AllocatorOf, ReaderOf, WriterOf},
        event::{emit, Event, SignalId},
        reactor::Signal,
        sample_storage::MyStorage,
    };

    use super::{executor::Executor, task::Task};

    type SharedStorage = Arc<Mutex<MyStorage>>;

    async fn emits_signal(mut storage: SharedStorage, signal_id: SignalId, value: usize) {
        println!("emits signal {value}");
        storage.put(&signal_id, value);
        emit(Event::SignalReadable(signal_id));
    }

    async fn mul2(
        mut storage: SharedStorage,
        signal_1_id: SignalId,
        signal_2_id: SignalId,
        out_signal_id: SignalId,
    ) {
        println!("mul2");
        let value_1 = Signal::new(storage.clone(), signal_1_id).await;
        let value_2 = Signal::new(storage.clone(), signal_2_id).await;

        storage.put(&out_signal_id, value_1 * value_2);
        emit(Event::SignalReadable(out_signal_id));
    }

    #[test]
    fn test_signaling() {
        let mut storage = SharedStorage::default();
        let first = storage.allocate();
        let second = storage.allocate();
        let result = storage.allocate();

        let mut executor = Executor::new();
        executor.spawn(Task::new(mul2(storage.clone(), first, second, result)));
        executor.spawn(Task::new(emits_signal(storage.clone(), first, 42)));
        executor.spawn(Task::new(emits_signal(storage.clone(), second, 1337)));
        executor.run_until_complete();

        assert_eq!(storage.get(&result), Some(42 * 1337));
    }

    async fn produce_value() -> usize {
        42
    }

    async fn prints_value() {
        let value = produce_value().await;
        println!("async works: {}", value);
    }

    #[test]
    fn test_executor_works() {
        let mut executor = Executor::new();
        executor.spawn(Task::new(prints_value()));
        executor.run_until_complete();
    }
}
