pub mod executor;
pub mod task;
pub mod waker;
pub mod adapters;

#[cfg(test)]
mod tests {
    use super::{executor::Executor, task::Task};

    async fn produce_value() -> usize {
        42
    }

    async fn prints_value() {
        let value = produce_value().await;
        println!("async works: {}", value);
    }

    async fn mul2(signal_1_id: SignalId, signal_2_id: SignalId, out_signal_id: SignalId) {
        let value_1 = signal(signal_1_id).await;
        let value_2 = signal(signal_2_id).await;
        // expected behaviour of make_future_for:
        // if signal is ready, return future::ready(value)
        // if not, make a new future which registers a watcher for this signal in storage

        // constrain!(signal_1_id * signal_2_id - out_signal_id);  // x * y - z, (signal_1_id, ...)

        emit(out_signal_id, value_1 * value_2);
    }

    #[test]
    fn test_executor_works() {
        let mut executor = Executor::new();
        executor.spawn(Task::new(prints_value()));
        executor.run_until_complete();
    }
}
