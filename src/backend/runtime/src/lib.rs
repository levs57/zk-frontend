pub mod event;
pub mod executor;
pub mod reactor;
pub mod storage;
pub mod task;
pub mod waker;

#[cfg(test)]
mod tests {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    use crate::{
        event::{
            signal::{Signal, SignalId},
            Event,
        },
        executor::Executor,
        storage::{AllocatorOf, ReaderOf, SharedStorage, WriterOf},
        task::Task,
    };

    async fn emits_signal(mut storage: SharedStorage, signal_id: SignalId, value: usize) {
        storage.put(&signal_id, value);
        Event::SignalReadable(signal_id).emit();
    }

    async fn mul2(
        mut storage: SharedStorage,
        signal_1_id: SignalId,
        signal_2_id: SignalId,
        out_signal_id: SignalId,
    ) {
        let value_1 = Signal::new(storage.clone(), signal_1_id).await;
        let value_2 = Signal::new(storage.clone(), signal_2_id).await;

        storage.put(&out_signal_id, value_1 * value_2);
        Event::SignalReadable(out_signal_id).emit();
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

    #[test]
    fn test_threading() {
        let mut storage = SharedStorage::default();
        let first = storage.allocate();
        let second = storage.allocate();
        let result = storage.allocate();

        let mut executor = Executor::new();
        executor.spawn(Task::new(mul2(storage.clone(), first, second, result)));
        executor.spawn(Task::new(emits_signal(storage.clone(), first, 42)));

        let sleep_duration = Duration::from_secs(5);
        let mut storage_clone = storage.clone();
        let start = Instant::now();
        thread::spawn(move || {
            thread::sleep(sleep_duration);
            storage_clone.put(&second, 1337);
            Event::SignalReadable(second).emit();
        });
        executor.run_until_complete();

        assert!(Instant::now() - start > sleep_duration);
        assert_eq!(storage.get(&result), Some(42 * 1337));
    }
}
