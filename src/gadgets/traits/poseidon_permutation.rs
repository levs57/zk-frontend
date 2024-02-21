use crate::circuit::{Advices, Circuit, HasSigtype, Sig, Signals, Variables};

use super::sponge::{TSpongePrivate, TSponge, SpongeAction};
use super::atoms::{Pow5, LinearCombination};

pub struct PoseidonSponge<C>
where
    C: Circuit + Signals + Variables,
    C::Config: HasSigtype<<C as Circuit>::F>,
{
    log: Vec<SpongeAction>,
    state: Vec<C::Sig<C::F>>,
    initial_capacity: C::Sig<C::F>,
    sep: usize,
    absorb_pos: usize,
    squeeze_pos: usize,
}

impl<C> TSpongePrivate<C> for PoseidonSponge<C>
where
    C: Circuit + PoseidonPermutation + Signals + Advices,
    C::Config: HasSigtype<<C as Circuit>::F>,
{
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
            initial_capacity: c.alloc_sig(),
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
        // c.advise_to_unassigned(|_| capacity, &(), &self.initial_capacity)
    }
}

impl<C> TSponge<C> for PoseidonSponge<C>
where
    C: Circuit + PoseidonPermutation + Signals + Advices,
    C::Config: HasSigtype<<C as Circuit>::F>,
{
    fn new(c: &mut C) -> Self {
        todo!()
    }
}

pub trait PoseidonPermutationImpl<C>
where
    C: Circuit + Signals + Pow5 + LinearCombination,
    C::Config: HasSigtype<<C as Circuit>::F>,
{
    fn poseidon_permutation(c: &mut C, inputs: Vec<C::Sig<C::F>>) -> Vec<C::Sig<C::F>>;
}

pub trait PoseidonPermutation
where
    Self: Circuit + Signals + Pow5 + LinearCombination,
    Self::Config: HasSigtype<<Self as Circuit>::F>,
{
    type ImplInstance: PoseidonPermutationImpl<Self>;
    fn poseidon(&mut self, inputs: Vec<Self::Sig<Self::F>>) -> Vec<Self::Sig<Self::F>> {
        Self::ImplInstance::poseidon_permutation(self, inputs)
    }
}
