use std::{collections::VecDeque, sync::{Arc, Mutex}, task::{Wake, Waker}};

use crate::task::TaskId;

pub(crate) struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<Mutex<VecDeque<TaskId>>>,
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_queue: Arc<Mutex<VecDeque<TaskId>>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.lock().unwrap().push_back(self.task_id);
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
