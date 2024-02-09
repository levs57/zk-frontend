use crate::circuit::{Circuit, Sig, StandardVariables};

pub trait TSponge<C: Circuit + StandardVariables> {
    fn new(c: &mut C) -> Self;
    fn absorb(&mut self, c: &mut C, input: Sig<C>);
    fn squeeze(&mut self, c: &mut C) -> Sig<C>;
}


pub trait PoseidonImpl<C : Circuit + StandardVariables> {
    type Sponge: TSponge<C>;
}

pub trait Poseidon<ImplInstance: PoseidonImpl<Self>> : Circuit + StandardVariables {
    fn new(&mut self) -> ImplInstance::Sponge {
        ImplInstance::Sponge::new(self)
    }

    fn absorb(&mut self, sponge: &mut ImplInstance::Sponge, input: Sig<Self>) {
        sponge.absorb(self, input)
    }

    fn squeeze(&mut self, sponge: &mut ImplInstance::Sponge) -> Sig<Self> {
        sponge.squeeze(self)
    }
}