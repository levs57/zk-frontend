// Probably need to move it into another crate to support importing it separately?

use crate::circuit::{Circuit, SVStruct, StandardVariables, IO};

impl<C: Circuit + StandardVariables + 'static, N : SVStruct<C> + 'static> IO<C> for N{
    type InputObject = Box<dyn Fn(&mut C, N::FStruct)>;

    type OutputObject = Box<dyn Fn(&mut C) -> N::FStruct>;

    fn inputize(self, circuit: &mut C) -> Self::InputObject {
        Box::new(move |c: &mut C, value: N::FStruct|self.write_to(c, value))
    }

    fn outputize(self, circuit: &mut C) -> Self::OutputObject {
        Box::new(move |c: &mut C|self.read_from(c))
    }
}