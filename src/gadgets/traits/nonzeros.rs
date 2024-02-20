use crate::circuit::{Circuit, Finalizes, Sig, StandardVariables};

pub trait NonzerosImpl<C: Circuit + StandardVariables + Finalizes> {
    fn enforce_nonzero(c: &mut C, x: Sig<C>);
}

pub trait Nonzeros : Circuit + StandardVariables + Finalizes {
    type INonzeros: NonzerosImpl<Self>;
    
    /// Pushes the signal in special container to perform batched nonzero check.
    /// Trait Finalizes signifies the necessity to perform the nonzero check during finalization.
    fn enforce_nonzero(&mut self, x: Sig<Self>) {
        Self::INonzeros::enforce_nonzero(self, x);
    }
}