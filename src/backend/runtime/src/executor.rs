use crossbeam_queue::SegQueue;
use std::{
    collections::BTreeMap,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use crate::{
    task::{Task, TaskId},
    waker::TaskWaker,
};

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<SegQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already exists");
        }
        self.task_queue.push(task_id);
    }

    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            println!("running {task_id:?}");
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists (sporadic wake)
            };
            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run_until_complete(&mut self) {
        while !self.tasks.is_empty() {
            self.run_ready_tasks();
        }
    }
}
