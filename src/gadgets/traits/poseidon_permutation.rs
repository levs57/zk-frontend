use std::marker::PhantomData;

use crate::circuit::{Circuit, Sig, StandardVariables};

use super::sponge::{TSpongePrivate, TSponge, SpongeAction};
use super::atoms::{Pow5, LinearCombination};

pub struct PoseidonSponge<C: Circuit + StandardVariables, F> {
    log: Vec<SpongeAction>,
    state: Vec<Sig<C>>,
    initial_capacity: Sig<C>,
    sep: usize,
    absorb_pos: usize,
    squeeze_pos: usize,
    _pd: PhantomData<F>,
}

impl<C: Circuit + StandardVariables + PoseidonPermutation, F> TSpongePrivate<C> for PoseidonSponge<C, F> {
    type DomainSeparator = usize;
    type Field = F;

    fn rate(&self) -> usize {
        self.state.len() - 1
    }

    fn absorb_pos(&self) -> usize {
        self.absorb_pos
    }

    fn set_absorb_pos(&mut self, new_pos: usize) {
        self.absorb_pos = new_pos
    }

    fn squeeze_pos(&self) -> usize {
        self.squeeze_pos
    }

    fn set_squeeze_pos(&mut self, new_pos: usize) {
        self.squeeze_pos = new_pos
    }

    fn new(c: &mut C, sep: Self::DomainSeparator, rate: usize) -> Self {
        let mut ret = Self {
            log: vec![],
            state: Vec::with_capacity(rate + 1),
            sep,
            absorb_pos: 0,
            squeeze_pos: 0,
            initial_capacity: c.alloc_sig().into(),
            _pd: PhantomData,
        };
        ret.state.push(ret.initial_capacity);
        ret
    }

    fn add_log(&mut self, action: SpongeAction) {
        self.log.push(action)
    }

    fn get_log(&self) -> Vec<SpongeAction> {
        self.log.clone()
    }

    fn tag_hasher(&self, items: Vec<u32>) -> Self::Field {
        todo!()
    }

    fn serialized_domain_separator(&self) -> Vec<u32> {
        vec![self.sep as u32]
    }

    fn read_rate_element(&self, offset: usize) -> Sig<C> {
        self.state[1 + offset]
    }

    fn add_rate_element(&mut self, offset: usize, value: Sig<C>) {
        self.state[1 + offset] = value
    }

    fn permute(&mut self, c: &mut C) {
        self.state = PoseidonPermutation::poseidon(c, self.state.clone())
    }

    fn initialize_capacity(&mut self, c: &mut C, capacity: Self::Field) {
        
    }
}

impl<C: Circuit + StandardVariables + PoseidonPermutation, F> TSponge<C> for PoseidonSponge<C, F> {
    fn new(c: &mut C) -> Self {
        todo!()
    }
}

pub trait PoseidonPermutationImpl<C : Circuit + StandardVariables + Pow5 + LinearCombination> {
    fn poseidon_permutation(c: &mut C, inputs: Vec<Sig<C>>) -> Vec<Sig<C>>;
}

pub trait PoseidonPermutation: Circuit + StandardVariables + Pow5 + LinearCombination {
    type ImplInstance: PoseidonPermutationImpl<Self>;
    fn poseidon(&mut self, inputs: Vec<Sig<Self>>) -> Vec<Sig<Self>> {
        Self::ImplInstance::poseidon_permutation(self, inputs)
    }
}
