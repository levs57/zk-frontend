use crate::circuit::{Circuit, HasSigtype, Sig, Signals};

pub trait Pow5
where
    Self: Circuit + Signals,
    Self::Config: HasSigtype<<Self as Circuit>::F>,
{
    fn pow5(&mut self, i: Self::Sig<Self::F>) -> Self::Sig<Self::F>;
}

pub trait LinearCombination
where
    Self: Circuit + Signals,
    Self::Config: HasSigtype<<Self as Circuit>::F>,
{
    fn lc(&mut self, coeffs: Vec<Self::F>, sigs: Vec<Self::Sig<Self::F>>) -> Self::Sig<Self::F>;
}
