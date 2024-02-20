use crate::circuit::{Circuit, HasSigtype, Sig, Signals, Unassigned, Variables};

use super::sponge::{TSpongePrivate, TSponge, SpongeAction};
use super::atoms::{Pow5, LinearCombination};

pub struct PoseidonSponge<C: Circuit + Signals + Variables + HasSigtype<<C as Circuit>::F>> {
    log: Vec<SpongeAction>,
    state: Vec<C::Sig<C::F>>,
    initial_capacity: Unassigned<C, C::F, C::Sig<C::F>>,
    sep: usize,
    absorb_pos: usize,
    squeeze_pos: usize,
}

impl<C: Circuit + PoseidonPermutation + HasSigtype<<C as Circuit>::F> + Signals> TSpongePrivate<C> for PoseidonSponge<C> {
    type DomainSeparator = usize;
    type Field = C::F;

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
            initial_capacity: c.alloc_sig_committed(),
        };
        ret.state.push(ret.initial_capacity.value());
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

    fn read_rate_element(&self, offset: usize) -> C::Sig<C::F> {
        self.state[1 + offset]
    }

    fn add_rate_element(&mut self, offset: usize, value: C::Sig<C::F>) {
        self.state[1 + offset] = value
    }

    fn permute(&mut self, c: &mut C) {
        self.state = PoseidonPermutation::poseidon(c, self.state.clone())
    }

    fn initialize_capacity(&mut self, c: &mut C, capacity: Self::Field) {

    }
}

impl<C: Circuit + PoseidonPermutation + HasSigtype<<C as Circuit>::F> + Signals> TSponge<C> for PoseidonSponge<C> {
    fn new(c: &mut C) -> Self {
        todo!()
    }
}

pub trait PoseidonPermutationImpl<C : Circuit + Pow5 + LinearCombination + Signals> {
    fn poseidon_permutation(c: &mut C, inputs: Vec<C::Sig<C::F>>) -> Vec<C::Sig<C::F>>;
}

pub trait PoseidonPermutation: Circuit + HasSigtype<<Self as Circuit>::F> + Pow5 + LinearCombination + Signals {
    type ImplInstance: PoseidonPermutationImpl<Self>;
    fn poseidon(&mut self, inputs: Vec<Self::Sig<Self::F>>) -> Vec<Self::Sig<Self::F>> {
        Self::ImplInstance::poseidon_permutation(self, inputs)
    }
}
