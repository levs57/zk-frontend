use std::{
    cmp::max,
    collections::{HashSet, VecDeque},
    marker::PhantomData,
    sync::Arc,
};

use ff::PrimeField;
use halo2curves::{group::Curve, msm::best_multiexp, CurveAffine};

use crate::circuit::{
    Circuit, CircuitGenericAdvices, CircuitGenericConstraints, CommitmentGroup, FieldConversion,
    FieldModulus, HasCGElt, HasCRhsAddr, HasSigAddr, HasVarAddr, Sig, StandardIOApi,
    StandardRoundApi, StandardVariables,
};

pub trait TypeConfig: 'static {
    type F: PrimeField + FieldModulus;
    type G: CurveAffine<ScalarExt = Self::F>;
}

pub struct ConstrEncoding<C: Circuit> {
    constr: Box<dyn FnMut(&mut C)>,
    deg: usize,
    _marker: PhantomData<C>,
}

pub enum Operation<C: Circuit, G> {
    Internal(Box<dyn FnMut(&mut C)>),
    Input(Box<dyn FnMut(&mut C, C::F)>),
    PublicValues(Box<dyn Fn(&C) -> Vec<C::F>>),
    Commitment(Box<dyn Fn(&C) -> G>),
}

// Depends on a field and a target cryptographic group G.
pub struct SimpleCircuit<T: TypeConfig> {
    signals: Vec<(Option<T::F>, usize)>,
    variables: Vec<Option<T::F>>,
    crhs: Vec<Option<T::F>>,

    current_pub_group: Option<SimplePubCommGroup<Self>>,
    current_priv_group: Option<SimplePrivCommGroup<Self, T>>,
    comm_keys: VecDeque<Vec<T::G>>,

    operations: VecDeque<Operation<Self, T::G>>,
    constraints: Vec<ConstrEncoding<Self>>,
}

impl<T: TypeConfig> SimpleCircuit<T> {
    pub fn init(mut comm_keys: VecDeque<Vec<T::G>>) -> Self {
        let ckey = Arc::new(comm_keys.pop_front().unwrap());

        let mut ret = Self {
            signals: vec![],
            variables: vec![],
            crhs: vec![],
            current_pub_group: None,
            current_priv_group: None,
            comm_keys,
            operations: VecDeque::new(),
            constraints: Vec::new(),
        };

        let current_pub_group = Some(SimplePubCommGroup::new(&mut ret, &()));
        let current_priv_group = Some(SimplePrivCommGroup::new(&mut ret, &ckey));

        ret.current_pub_group = current_pub_group;
        ret.current_priv_group = current_priv_group;

        ret
    }
}

pub enum SimpleRaw {
    Var(usize),
    Sig(usize),
    CRhs(usize),
}

impl<T: TypeConfig> Circuit for SimpleCircuit<T> {
    type F = T::F;
    type RawAddr = SimpleRaw;
}

/* Implementing addresses */
impl<T: TypeConfig> StandardVariables for SimpleCircuit<T> {
    type SA = usize;
    type VA = usize;
    type CA = usize;
}

impl<T: TypeConfig> FieldConversion<T::F> for SimpleCircuit<T> {
    fn felt(value: T::F) -> Self::F {
        value
    }
}

impl<T: TypeConfig> HasSigAddr<usize> for SimpleCircuit<T> {
    type Value = T::F;

    fn alloc_sig(&mut self) -> usize {
        let l = self.signals.len();
        self.signals.push((None, 0));
        l
    }

    fn read_sig(&self, addr: usize) -> Self::Value {
        self.signals[addr].0.unwrap()
    }

    fn write_sig(&mut self, addr: usize, value: Self::Value) {
        match self.signals[addr].0.replace(value) {
            None => (),
            Some(_) => panic!(),
        }
    }

    fn into_raw_addr(&self, addr: usize) -> Self::RawAddr {
        SimpleRaw::Sig(addr)
    }

    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> usize {
        match raw {
            SimpleRaw::Sig(addr) => addr,
            _ => panic!(),
        }
    }
}

impl<T: TypeConfig> HasVarAddr<usize> for SimpleCircuit<T> {
    type Value = T::F;

    fn alloc_var(&mut self) -> usize {
        let l = self.variables.len();
        self.variables.push(None);
        l
    }

    fn read_var(&self, addr: usize) -> Self::Value {
        self.variables[addr].unwrap()
    }

    fn write_var(&mut self, addr: usize, value: Self::Value) {
        match self.variables[addr].replace(value) {
            None => (),
            Some(_) => panic!(),
        }
    }

    fn into_raw_addr(&self, addr: usize) -> Self::RawAddr {
        SimpleRaw::Var(addr)
    }

    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> usize {
        match raw {
            SimpleRaw::Var(addr) => addr,
            _ => panic!(),
        }
    }
}

impl<T: TypeConfig> HasCRhsAddr<usize> for SimpleCircuit<T> {
    type Value = T::F;

    fn alloc_crhs(&mut self) -> usize {
        let l = self.crhs.len();
        self.crhs.push(None);
        l
    }

    fn read_crhs(&self, addr: usize) -> Self::Value {
        self.crhs[addr].unwrap()
    }

    fn write_crhs(&mut self, addr: usize, value: Self::Value) {
        match self.crhs[addr].replace(value) {
            None => (),
            Some(_) => panic!(),
        }
    }

    fn into_raw_addr(&self, addr: usize) -> Self::RawAddr {
        SimpleRaw::CRhs(addr)
    }

    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> usize {
        match raw {
            SimpleRaw::CRhs(addr) => addr,
            _ => panic!(),
        }
    }
}

pub struct SimplePubCommGroup<C: Circuit + StandardVariables + 'static> {
    signals: HashSet<C::SA>,
}

impl<C: Circuit + StandardVariables + 'static> CommitmentGroup<C> for SimplePubCommGroup<C> {
    type CommitmentObject = Box<dyn Fn(&mut C) -> Vec<C::F>>;
    type Params = ();

    fn new(_circuit: &mut C, _: &()) -> Self {
        Self {
            signals: HashSet::new(),
        }
    }

    fn commit(self, _circuit: &mut C) -> Self::CommitmentObject {
        let sigvec: Vec<_> = self.signals.into_iter().collect();
        Box::new(move |circuit: &mut C| {
            let ret: Vec<_> = (*sigvec)
                .into_iter()
                .map(|sig| circuit.read_sig(*sig))
                .collect();
            ret
        })
    }
}

pub struct SimplePrivCommGroup<C: Circuit + StandardVariables + 'static, T: TypeConfig> {
    signals: HashSet<C::SA>,
    params: Arc<Vec<T::G>>,
}

impl<C: Circuit + StandardVariables, T: TypeConfig<F = C::F>> CommitmentGroup<C>
    for SimplePrivCommGroup<C, T>
{
    type CommitmentObject = Box<dyn Fn(&mut C) -> T::G>;
    type Params = Arc<Vec<T::G>>;

    fn new(_circuit: &mut C, params: &Self::Params) -> Self {
        Self {
            signals: HashSet::new(),
            params: params.clone(),
        }
    }

    fn commit(self, _circuit: &mut C) -> Self::CommitmentObject {
        let Self { signals, params } = self;
        let signals: Vec<_> = signals.into_iter().collect();
        Box::new(move |c: &mut C| {
            let values: Vec<_> = (*signals).into_iter().map(|i| c.read_sig(*i)).collect();
            let bases = params.as_slice();
            best_multiexp(&values, bases).to_affine()
        })
    }
}

impl<T: TypeConfig> CircuitGenericAdvices for SimpleCircuit<T> {
    fn advise<
        I: crate::circuit::SVStruct<Self>,
        O: crate::circuit::SVStruct<Self>,
        Fun: Fn(I::FStruct) -> O::FStruct + 'static,
    >(
        &mut self,
        f: Fun,
        inp: I,
    ) -> O {
        let output = O::alloc_to(self);
        let op = Box::new(move |c: &mut Self| {
            output.write_to(c, f(inp.read_from(c)));
        });
        self.operations.push_back(Operation::Internal(op));
        output
    }

    fn advise_variadic<
        I: crate::circuit::SVStruct<Self>,
        O: crate::circuit::SVStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> Vec<O::FStruct> + 'static,
    >(
        &mut self,
        f: Fun,
        num_outputs: usize,
        inp: Vec<I>,
    ) -> Vec<O> {
        let output = vec![O::alloc_to(self); num_outputs]; // This one goes into closure
        let ret = output.clone(); // And this to output.
        let op = Box::new(move |c: &mut Self| {
            let inp: Vec<_> = inp.iter().map(|s| s.read_from(c)).collect();
            let out = f(&inp);
            assert!(out.len() == num_outputs);
            output
                .clone()
                .iter()
                .zip(out.into_iter())
                .map(|(target, value)| target.write_to(c, value))
                .count();
        });
        self.operations.push_back(Operation::Internal(op));
        ret
    }
}

impl<T: TypeConfig> CircuitGenericConstraints for SimpleCircuit<T> {
    fn enforce<
        I: crate::circuit::SigStruct<Self>,
        O: crate::circuit::CRhsStruct<Self>,
        Fun: Fn(I::FStruct) -> O::FStruct + 'static,
    >(
        &mut self,
        f: Fun,
        deg: usize,
        inp: I,
    ) -> O {
        let output = O::alloc_to(self);

        inp.to_raw_addr(&self)
            .into_iter()
            .map(|raw| {
                let idx = <Self as HasSigAddr<_>>::try_parse_raw_addr(self, raw);
                self.signals[idx].1 = max(self.signals[idx].1, deg);
            })
            .count(); // Iterate through addresses and increase metadata parameter degree.

        let constr = Box::new(move |c: &mut Self| {
            output.write_to(c, f(inp.read_from(c)));
        });
        let constr = ConstrEncoding {
            constr,
            deg,
            _marker: PhantomData,
        };
        self.constraints.push(constr);
        output
    }

    fn enforce_variadic<
        I: crate::circuit::SigStruct<Self>,
        O: crate::circuit::CRhsStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> Vec<O::FStruct> + 'static,
    >(
        &mut self,
        f: Fun,
        deg: usize,
        num_outputs: usize,
        inp: Vec<I>,
    ) -> Vec<O> {
        let output = vec![O::alloc_to(self); num_outputs]; // This one goes into closure
        let ret = output.clone(); // And this to output.

        let raws: Vec<_> = inp
            .iter()
            .map(|st| st.to_raw_addr(&self))
            .flatten()
            .map(|raw| <Self as HasSigAddr<_>>::try_parse_raw_addr(&self, raw))
            .collect();

        raws.into_iter()
            .map(|idx| {
                self.signals[idx].1 = max(self.signals[idx].1, deg);
            })
            .count(); // Iterate through addresses and increase metadata parameter degree.

        let constr = Box::new(move |c: &mut Self| {
            let inp: Vec<_> = inp.iter().map(|s| s.read_from(c)).collect();
            let out = f(&inp);
            assert!(out.len() == num_outputs);
            output
                .clone()
                .iter()
                .zip(out.into_iter())
                .map(|(target, value)| target.write_to(c, value))
                .count();
        });

        let constr = ConstrEncoding {
            constr,
            deg,
            _marker: PhantomData,
        };
        self.constraints.push(constr);
        ret
    }
}

impl<T: TypeConfig> StandardRoundApi for SimpleCircuit<T> {
    type PubCommGroup = SimplePubCommGroup<Self>;
    type PrivCommGroup = SimplePrivCommGroup<Self, T>;

    fn next_round(
        &mut self,
    ) -> (
        <Self::PubCommGroup as CommitmentGroup<Self>>::CommitmentObject,
        <Self::PrivCommGroup as CommitmentGroup<Self>>::CommitmentObject,
    ) {
        let ckey = Arc::new(self.comm_keys.pop_front().unwrap());
        let new_pub_group = SimplePubCommGroup::new(self, &());
        let new_priv_group = SimplePrivCommGroup::new(self, &ckey);

        let pub_group = self.current_pub_group.replace(new_pub_group).unwrap();
        let priv_group = self.current_priv_group.replace(new_priv_group).unwrap();

        let pubs = pub_group.commit(self);
        let priv_comm = priv_group.commit(self);

        (pubs, priv_comm)
    }
}

impl<T: TypeConfig> StandardIOApi for SimpleCircuit<T> {
    fn public_output(&mut self, sig: Sig<Self>) {
        self.current_priv_group.as_mut().unwrap().remove(sig);
        self.current_pub_group.as_mut().unwrap().add(sig);
    }

    fn public_input(&mut self) -> Sig<Self> {
        let sig = self.alloc_sig().into();
        self.current_pub_group.as_mut().unwrap().add(sig);
        sig
    }

    fn private_input(&mut self) -> Sig<Self> {
        let sig = self.alloc_sig().into();
        self.current_priv_group.as_mut().unwrap().add(sig);
        sig
    }
}

impl<T: TypeConfig> HasCGElt<usize, SimpleCircuit<T>> for SimplePubCommGroup<SimpleCircuit<T>> {
    fn add(&mut self, sig: crate::circuit::Signal<usize, SimpleCircuit<T>>) {
        assert!(self.signals.insert(sig.addr()));
    }

    fn remove(&mut self, sig: crate::circuit::Signal<usize, SimpleCircuit<T>>) {
        assert!(self.signals.remove(&sig.addr()));
    }
}

impl<T: TypeConfig> HasCGElt<usize, SimpleCircuit<T>> for SimplePrivCommGroup<SimpleCircuit<T>, T> {
    fn add(&mut self, sig: crate::circuit::Signal<usize, SimpleCircuit<T>>) {
        assert!(self.signals.insert(sig.addr()));
    }

    fn remove(&mut self, sig: crate::circuit::Signal<usize, SimpleCircuit<T>>) {
        assert!(self.signals.remove(&sig.addr()));
    }
}
