pub mod executor;
pub mod task;

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

    async fn add_two_nodes(node1, node2, out_node) {
        let round_lock = wait_for_round(3).await;
        let value1 = node1.await;
        let value2 = node2.await;

        out_node.emit_event_ready(value1 + value2);
    }

    #[test]
    fn test_executor_works() {
        let mut executor = Executor::new();
        executor.spawn(Task::new(prints_value()));
        executor.run_until_complete();

        executor.emit_bump_round();
        executor.spawn(Task::new(prints_value()));
        executor.run_until_complete()
    }
}
