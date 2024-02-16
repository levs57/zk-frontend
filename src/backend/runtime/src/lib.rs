pub mod adapters;
pub mod event;
pub mod executor;
pub mod reactor;
pub mod task;
pub mod waker;

#[cfg(test)]
mod tests {
    use crate::event::{emit, Event, SignalId};

    use super::{executor::Executor, task::Task};

    async fn produce_value() -> usize {
        42
    }

    async fn prints_value() {
        let value = produce_value().await;
        println!("async works: {}", value);
    }

    async fn mul2(signal_1_id: SignalId, signal_2_id: SignalId, out_signal_id: SignalId) {
        let value_1 = SignalFuture(signal_1_id).await;
        let value_2 = SignalFuture(signal_2_id).await;

        storage.put(&out_signal_id, value_1 * value_2);
        emit(Event::SignalReadable(out_signal_id));
    }

    #[test]
    fn test_executor_works() {
        let mut executor = Executor::new();
        executor.spawn(Task::new(prints_value()));
        executor.run_until_complete();
    }
}
