use std::{
    sync::Arc,
    task::{Wake, Waker},
};

use crossbeam_queue::SegQueue;

use crate::task::TaskId;

/// A `Waker`-flavoured struct used by `Executor` and `Reactor`
/// to wake up paused tasks.
///
/// Should only be created inside an executor instance.
pub(crate) struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<SegQueue<TaskId>>,
}

impl TaskWaker {
    pub(crate) fn new(task_id: TaskId, task_queue: Arc<SegQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        println!("waking {:?}", self.task_id);
        self.task_queue.push(self.task_id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
