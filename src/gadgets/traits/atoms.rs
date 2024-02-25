use crate::circuit::{Circuit, HasSigtype, Sig, Signals};

pub trait Pow5
where
    Self: Circuit + Signals,
    Self::Config: HasSigtype<<Self as Circuit>::F>,
{
    fn pow5(&mut self, i: Sig<Self, Self::F>) -> Sig<Self, Self::F>;
}

pub trait LinearCombination
where
    Self: Circuit + Signals,
    Self::Config: HasSigtype<<Self as Circuit>::F>,
{
    fn lc(&mut self, coeffs: Vec<Self::F>, sigs: Vec<Sig<Self, Self::F>>) -> Sig<Self, Self::F>;
}
