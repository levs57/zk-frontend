use crate::reactor::Reactor;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct SignalId(pub usize);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Event {
    SignalReadable(SignalId),
}

pub fn emit(event: Event) {
    // TODO: fancy pub/sub here: buffer emitted events into separate queue, react later
    println!("emitting {event:?}");
    Reactor::get().react(event);
}
