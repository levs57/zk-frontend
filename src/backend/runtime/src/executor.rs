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

/// Top-level executor for asynchronous tasks.
///
/// This is used to spawn and execute the runtime phase
/// of the circuit. It uses concurrent (async) functions under
/// the hood, even though circuit runtime is primarily CPU-bound.
/// This is justified, since many workflows require some sort of
/// cooperation between the advices.
///
/// An example of such a workflow would be the division gadget. It is easily
/// seen that in order to obtain the inverses for a set of variables
/// one has to only invert their product. So it is beneficial to collect
/// as many variables as possible, and then perform a division only once.
/// This suggests that a gadget using variable's inverse should not compute
/// it on its one, but rather delegate it to some other subroutine. After this
/// delegation, the gadget is free to use a variable symbolically (i. e. without
/// reading its value), pausing its execution when the value is actually required.
/// The subroutine will later compute the value, resuming gadget's execution.
///
/// Another example is the concept of a round in proof systems like ProtoStar. This
/// also fits neatly into the framework of pausable functions. Again, a gadget is
/// ran for a while, paused until the next round begins, and then resumed.
///
/// Circuit IO is yet another example, somewhat similar to the first one. An entire
/// coprocessor circuit (such as CycleFold) could be instantiated to cooperatively
/// perform expensive (non-native) operations.
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

    /// Spawn a task inside an executor.
    ///
    /// The task will be executed concurrently with other tasks.
    /// This function will panic if a task with the same id already
    /// exisis inside the executor.
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

    /// Block on currently spawned tasks, driving them to completion.
    pub fn run_until_complete(&mut self) {
        while !self.tasks.is_empty() {
            self.run_ready_tasks();
        }
    }
}
