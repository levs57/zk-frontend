use crate::circuit::{Circuit, Sig, StandardVariables};

pub trait PoseidonPermutationImpl<C : Circuit + StandardVariables> {
    fn poseidon_permutation(c: &mut C, inputs: Vec<Sig<C>>) -> Vec<Sig<C>>;
}

pub trait PoseidonPermutation: Circuit + StandardVariables {
    type ImplInstance: PoseidonPermutationImpl<Self>;
    fn poseidon(&mut self, inputs: Vec<Sig<Self>>) -> Vec<Sig<Self>> {
        Self::ImplInstance::poseidon_permutation(self, inputs)
    }
}
