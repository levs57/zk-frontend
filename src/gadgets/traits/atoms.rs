use crate::circuit::{Circuit, Sig, StandardVariables};

pub trait Pow5: Circuit + StandardVariables {
    fn pow5(&mut self, i: Sig<Self>) -> Sig<Self>;
}

pub trait LinearCombination: Circuit + StandardVariables {
    fn lc(&mut self, coeffs: Vec<Self::F>, sigs: Vec<Sig<Self>>) -> Sig<Self>;
}
