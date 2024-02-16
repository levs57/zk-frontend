use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll},
};

/// A tag identifying tasks inside an executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(usize);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// A tagged wrapper around `Future`.
///
/// Marks top-level futures. Can be spawned and ran by an `Executor`.
pub struct Task {
    pub(crate) id: TaskId,
    pub(crate) future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    pub(crate) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
