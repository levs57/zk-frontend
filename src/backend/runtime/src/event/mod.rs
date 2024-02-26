pub mod signal;

use crate::reactor::Reactor;

use self::signal::SignalId;

/// A marker that some side effect has taken action.
///
/// To actually emit the event, use the `Event::emit()` method.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Event {
    /// A signal became readable.
    SignalReadable(SignalId),
}

impl Event {
    /// Emit an event.
    ///
    /// This is used to notify the paused tasks
    /// that some event in the outside world has occured.
    ///
    /// May buffer events instead of processing them right away.
    /// Can be called from any thread.
    ///
    /// This method (somewhat magically) notifies the reactor about the event
    /// under the hood. Passing `Reactor` through the stack as a method argument
    /// would require users to actually care about this internal abstraction, on top
    /// of being cumbersome.
    pub fn emit(self) {
        // TODO: fancy pub/sub here: buffer emitted events into separate queue, react later
        println!("emitting {self:?}");
        Reactor::get().react(self);
    }
}
