use crate::circuit::{Circuit, HasSigtype, Sig, Signals};

pub trait Pow5: Circuit + HasSigtype<<Self as Circuit>::F> + Signals {
    fn pow5(&mut self, i: Self::Sig<Self::F>) -> Self::Sig<Self::F>;
}

pub trait LinearCombination: Circuit + HasSigtype<<Self as Circuit>::F> + Signals {
    fn lc(&mut self, coeffs: Vec<Self::F>, sigs: Vec<Self::Sig<Self::F>>) -> Self::Sig<Self::F>;
}
