use crate::circuit::{Circuit, HasSigtype, Sig};

pub trait NonzerosImpl<C: Circuit + HasSigtype<<C as Circuit>::F>> {
    fn enforce_nonzero(c: &mut C, x: Sig<C, C::F>);
}

pub trait Nonzeros : Circuit + HasSigtype<<Self as Circuit>::F> {
    type INonzeros: NonzerosImpl<Self>;
    
    fn enforce_nonzero(&mut self, x: Sig<Self, Self::F>) {
        Self::INonzeros::enforce_nonzero(self, x);
    }
}