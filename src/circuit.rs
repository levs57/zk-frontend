use std::{collections::VecDeque, marker::PhantomData};

use ff::PrimeField;
use macros::make_tuple_impls;

/// Hub trait.
pub trait Circuit : Sized + 'static {
    type F : PrimeField;
    type RawAddr; // Optimally, this primitive should not be used to access values themselves, only metadata.
                  // It MUST be collision-free.
}

/// Trait enabling reading and writing of a particular sigvar structure. 
/// Default implementation is setter / getter (but could be node label if we want to serialize).
pub trait IO<C : Circuit> {
    type InputObject;
    type OutputObject;

    /// Makes sigvar structure writeable. Can be used for private inputs.
    fn inputize(self, circuit: &mut C) -> Self::InputObject;
    /// Makes sigvar structure readable. Can be used for private outputs.
    /// Usage for public outputs is typically unnecessary, as they are added to public commitment group.
    fn outputize(self, circuit: &mut C) -> Self::OutputObject;
}

/// Standard nodes defined over the base field.
pub trait StandardVariables : Circuit + 
        HasSigAddr<Self::SA, Value = <Self as Circuit>::F> + 
        HasVarAddr<Self::VA, Value = <Self as Circuit>::F> +
        HasCRhsAddr<Self::CA, Value = <Self as Circuit>::F>
    where
        Sig<Self> : IO<Self>,
        Var<Self> : IO<Self>,
{
    type SA : Copy + 'static;
    type VA : Copy + 'static;
    type CA : Copy + 'static; 
}

pub type Sig<C> = Signal<<C as StandardVariables>::SA, C>; 
pub type Var<C> = Variable<<C as StandardVariables>::VA, C>;


/// This trait describes default way of handling commitment groups.
/// It is as follows: there are 2 groups for each round, one reserved for public signals,
/// and one for private signals. Each new created signal falls into private group by default,
/// but can be ejected into public one. 
pub trait StandardRoundApi : Circuit + StandardVariables where Sig<Self> : IO<Self>, Var<Self> : IO<Self> {
    // The fact that I must copy where clauses everywhere is supremely dumb.
    
    type PubCommGroup: CommitmentGroup<Self> + HasCGElt<Self::SA, Self>;
    type PrivCommGroup : CommitmentGroup<Self> + HasCGElt<Self::SA, Self>;

    /// Expected behavior is that circuit manages two such groups internally and then spawns new ones when round is finalized.
    fn next_round(&mut self) -> (
        <Self::PubCommGroup as CommitmentGroup<Self>>::CommitmentObject,
        <Self::PrivCommGroup as CommitmentGroup<Self>>::CommitmentObject
    );

    /// Moves element from the default private commitment group to the corresponding public.
    fn public_output(&mut self, sig: Sig<Self>);

    /// Creates an element in the public commitment group and outputs writer.
    fn public_input(&mut self) -> Sig<Self>;

    /// Creates an element in the private commitment group.
    fn private_input(&mut self) -> Sig<Self>;
}

pub trait CircuitGenericAdvices : Circuit {
    fn advise<I: SVStruct<Self>, O: SVStruct<Self>, Fun: Fn(I::FStruct) -> O::FStruct + 'static>(&mut self, f: Fun, inp: I) -> O;

    fn advise_variadic<
        I: SVStruct<Self>,
        O: SVStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> Vec<O::FStruct> + 'static,
    > (&mut self, f: Fun, num_outputs: usize, inp: Vec<I>) -> Vec<O>;

}

pub trait CircuitGenericConstraints : Circuit {
    fn enforce<I: SigStruct<Self>, O: CRhsStruct<Self>, Fun: Fn(I::FStruct) -> O::FStruct + 'static>(&mut self, f: Fun, deg: usize, inp: I) -> O;

    fn enforce_variadic<
        I: SigStruct<Self>,
        O: CRhsStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> Vec<O::FStruct> + 'static,
    > (&mut self, f: Fun, deg: usize, num_outputs: usize, inp: Vec<I>) -> Vec<O>;

}

pub trait CommitmentGroup<C: Circuit> : Sized {
    type CommitmentObject;
    type Params;
    /// Creates an empty commitment group and registers it.
    fn new(circuit: &mut C, params: &Self::Params) -> Self;

    // Actually, an operation. Could be emulated by variadic advice if you are brave enough, separated for clarity.

    fn commit(self, circuit: &mut C) -> Self::CommitmentObject;
}

pub trait HasCGElt<SigAddr : Copy + 'static, C: Circuit> : CommitmentGroup<C> where C : HasSigAddr<SigAddr>{
    fn add(&mut self, sig : Signal<SigAddr, C>);
    fn remove(&mut self, sig: Signal<SigAddr, C>);
}

/// This trait allows to convert elements of type T to the ground field of the circuit.
pub trait FieldConversion<T> : Circuit {
    fn felt(value: T) -> Self::F;
}

/// Value of type T that can be held by the circuit.
pub trait HasVarAddr<VarAddr : Copy> : Circuit + Sized {
    type Value;
    fn alloc_var(&mut self) -> VarAddr;
    fn read_var(&self, addr: VarAddr) -> Self::Value;
    fn write_var(&mut self, addr: VarAddr, value: Self::Value); 
    fn into_raw_addr(&self, addr: VarAddr) -> Self::RawAddr;
    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> VarAddr;
}

pub trait HasSigAddr<SigAddr : Copy + 'static> : Sized + FieldConversion<Self::Value> {
    type Value;
    fn alloc_sig(&mut self) -> SigAddr;
    fn read_sig(&self, addr: SigAddr) -> Self::Value;
    fn write_sig(&mut self, addr: SigAddr, value: Self::Value);
    fn into_raw_addr(&self, addr: SigAddr) -> Self::RawAddr;
    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> SigAddr;

}

// pub trait HasSigValue<T> : Circuit + Sized + HasSigAddr<Self::SigAddr, Value = Self> {
//     type SigAddr : Copy;
// }

// pub trait IsSigValue<C : HasSigValue<Self>> : Sized {
//     type SigAddr;
// }
// impl<T, C : HasSigValue<T>> IsSigValue<C> for T {
//     type SigAddr = C::SigAddr;
// }


pub trait HasCRhsAddr<CRhsAddr : Copy + 'static> : Sized + FieldConversion<Self::Value> {
    type Value;
    fn alloc_crhs(&mut self) -> CRhsAddr;
    fn read_crhs(&self, addr: CRhsAddr) -> Self::Value;
    fn write_crhs(&mut self, addr: CRhsAddr, value: Self::Value); 
    fn into_raw_addr(&self, addr: CRhsAddr) -> Self::RawAddr;
    fn try_parse_raw_addr(&self, raw: Self::RawAddr) -> CRhsAddr;
}

// pub trait HasCRhsValue<T> : Circuit + Sized + HasCRhsAddr<Self::CRhsAddr, Value = Self> {
//     type CRhsAddr : Copy;
// }

// pub trait IsCRhsValue<C : HasCRhsValue<Self>> : Sized {
//     type CRhsAddr;
// }
// impl<T, C : HasCRhsValue<T>> IsCRhsValue<C> for T {
//     type CRhsAddr = C::CRhsAddr;
// }

pub struct Variable<VarAddr : Copy + 'static, C : HasVarAddr<VarAddr>> {
    addr: VarAddr,
    _marker: PhantomData<C>,
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Clone for Variable<VarAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Copy for Variable<VarAddr, C> {}


impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> From<VarAddr> for Variable<VarAddr, C> {
    fn from(value: VarAddr) -> Self {
        Variable {addr : value, _marker : PhantomData}
    }
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Variable<VarAddr, C> {
    pub fn addr(self) -> VarAddr {
        self.addr
    }
}

pub struct Signal<SigAddr : Copy + 'static, C : HasSigAddr<SigAddr>> {
    addr: SigAddr,
    _marker: PhantomData<C>,
}

impl <SigAddr : Copy + 'static, C : HasSigAddr<SigAddr>> Clone for Signal<SigAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <SigAddr : Copy + 'static, C : HasSigAddr<SigAddr>> Copy for Signal<SigAddr, C> {}


impl <SigAddr : Copy + 'static, C : HasSigAddr<SigAddr>> From<SigAddr> for Signal<SigAddr, C> {
    fn from(value: SigAddr) -> Self {
        Signal {addr : value, _marker : PhantomData}
    }
}

impl <SigAddr: Copy + 'static, C : HasSigAddr<SigAddr>> Signal<SigAddr, C> {
    pub fn addr(self) -> SigAddr {
        self.addr
    }
}
pub struct ConstrRhs<CRhsAddr : Copy + 'static, C : HasCRhsAddr<CRhsAddr>> {
    addr: CRhsAddr,
    _marker: PhantomData<C>,
}

impl <CRhsAddr : Copy + 'static, C : HasCRhsAddr<CRhsAddr>> Clone for ConstrRhs<CRhsAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <CRhsAddr : Copy + 'static, C : HasCRhsAddr<CRhsAddr>> Copy for ConstrRhs<CRhsAddr, C> {}


impl <CRhsAddr : Copy + 'static, C : HasCRhsAddr<CRhsAddr>> From<CRhsAddr> for ConstrRhs<CRhsAddr, C> {
    fn from(value: CRhsAddr) -> Self {
        ConstrRhs {addr : value, _marker : PhantomData}
    }
}

impl <CRhsAddr : Copy + 'static, C : HasCRhsAddr<CRhsAddr>> ConstrRhs<CRhsAddr, C> {
    pub fn addr(self) -> CRhsAddr {
        self.addr
    }
}


/// This must represent static structures built from in-circuit runtime values.
pub trait NodeStruct<C : Circuit> : Copy + 'static {
    type FStruct;

    fn alloc_to(c: &mut C) -> Self;
    fn read_from(self, c: &C) -> Self::FStruct;
    fn write_to(self, c: &mut C, value: Self::FStruct); 
    fn to_raw_addr(&self, c: &C) -> VecDeque<C::RawAddr>; // This and below = nuclear options. 
    /// Pops elements from the queue and attempts to parse them as a struct.
    fn try_from_raw_addr(c: &C, raws: &mut VecDeque<C::RawAddr>) -> Self; // Use only to access metadata.
}

/// This must represent static structures built from signals and variables.
pub trait SVStruct<C: Circuit> : NodeStruct<C> {}

/// This must represent object built from signals.
pub trait SigStruct<C: Circuit> : SVStruct<C> {}

/// This must represent object built from constraint values.
pub trait CRhsStruct<C: Circuit> : NodeStruct<C> {}


// -- BASE IMPLS FOR VAR AND SIG

impl<VarAddr : Copy, C : HasVarAddr<VarAddr>> NodeStruct<C> for Variable<VarAddr, C> {
    type FStruct = C::Value;

    fn alloc_to(c: &mut C) -> Self {
        c.alloc_var().into()
    }

    fn read_from(self, c: &C) -> Self::FStruct {
        c.read_var(self.addr())
    }

    fn write_to(self, c: &mut C, value: Self::FStruct) {
        c.write_var(self.addr(), value)
    }

    fn to_raw_addr(&self, c: &C) -> VecDeque<<C>::RawAddr> {
        let mut ret = VecDeque::new();
        ret.push_back(c.into_raw_addr(self.addr()));
        ret
    }

    fn try_from_raw_addr(c: &C, raws: &mut VecDeque<C::RawAddr>) -> Self {
        let raw = raws.pop_front().unwrap();
        c.try_parse_raw_addr(raw).into()
    }
}

impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> NodeStruct<C> for Signal<SigAddr, C> {
    type FStruct = C::Value;

    fn alloc_to(c: &mut C) -> Self {
        c.alloc_sig().into()
    }

    fn read_from(self, c: &C) -> Self::FStruct {
        c.read_sig(self.addr())
    }

    fn write_to(self, c: &mut C, value: Self::FStruct) {
        c.write_sig(self.addr(), value)
    }

    fn to_raw_addr(&self, c: &C) -> VecDeque<<C>::RawAddr> {
        let mut ret = VecDeque::new();
        ret.push_back(c.into_raw_addr(self.addr()));
        ret
    }

    fn try_from_raw_addr(c: &C, raws: &mut VecDeque<C::RawAddr>) -> Self {
        let raw = raws.pop_front().unwrap();
        c.try_parse_raw_addr(raw).into()
    }
}

impl<CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> NodeStruct<C> for ConstrRhs<CRhsAddr, C> {
    type FStruct = C::Value;

    fn alloc_to(c: &mut C) -> Self {
        c.alloc_crhs().into()
    }

    fn read_from(self, c: &C) -> Self::FStruct {
        c.read_crhs(self.addr())
    }

    fn write_to(self, c: &mut C, value: Self::FStruct) {
        c.write_crhs(self.addr(), value)
    }

    fn to_raw_addr(&self, c: &C) -> VecDeque<<C>::RawAddr> {
        let mut ret = VecDeque::new();
        ret.push_back(c.into_raw_addr(self.addr()));
        ret
    }

    fn try_from_raw_addr(c: &C, raws: &mut VecDeque<C::RawAddr>) -> Self {
        let raw = raws.pop_front().unwrap();
        c.try_parse_raw_addr(raw).into()
    }
}

impl<VarAddr : Copy, C : HasVarAddr<VarAddr>> SVStruct<C> for Variable<VarAddr, C> {}
impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> SVStruct<C> for Signal<SigAddr, C> {}

impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> SigStruct<C> for Signal<SigAddr, C> {}

impl<CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> CRhsStruct<C> for ConstrRhs<CRhsAddr, C> {}

// -- ARRAYS

impl<C : Circuit, T: NodeStruct<C>, const N: usize> NodeStruct<C> for [T; N] {
    type FStruct = [T::FStruct; N];

    fn alloc_to(c: &mut C) -> Self {
        let mut ret = Vec::with_capacity(N);
        for _ in 0..N {
            ret.push(T::alloc_to(c).into())
        }
        ret.try_into().unwrap_or_else(|_|panic!())
    }

    fn read_from(self, c: &C) -> Self::FStruct {
        let mut ret = Vec::with_capacity(N);
        self.into_iter().map(|t|
            ret.push(t.read_from(c))
        ).count();
        ret.try_into().unwrap_or_else(|_|panic!())
    }

    fn write_to(self, c: &mut C, value: Self::FStruct) {
        self.into_iter().zip(value.into_iter()).map(|(t, v)|t.write_to(c, v)).count();
    }

    fn to_raw_addr(&self, c: &C) -> VecDeque<<C as Circuit>::RawAddr> {
        let mut ret = VecDeque::new();
        for i in 0..N {
            ret.append(&mut self[i].to_raw_addr(c))
        }
        ret
    }

    fn try_from_raw_addr(c: &C, raws: &mut VecDeque<<C as Circuit>::RawAddr>) -> Self {
        let mut ret = Vec::with_capacity(N);
        for i in 0..N {
            ret.push(T::try_from_raw_addr(c, raws));
        }
        ret.try_into().unwrap_or_else(|_|panic!())
    }



}

impl<C: Circuit, T: SVStruct<C>, const N: usize> SVStruct<C> for [T; N] {}
impl<C: Circuit, T: SigStruct<C>, const N: usize> SigStruct<C> for [T; N] {}
impl<C: Circuit, T: CRhsStruct<C>, const N: usize> CRhsStruct<C> for [T; N] {}

// impl<C: Circuit, T1: NodeStruct<C>, T2: NodeStruct<C>> NodeStruct<C> for (T1, T2) {
//     type FStruct = (T1::FStruct, T2::FStruct);
//     fn alloc_to(c: &mut C) -> Self {
//         (T1::alloc_to(c), T2::alloc_to(c))
//     }
//     fn read_from(self, c: &C) -> Self::FStruct {
//         (self.0.read_from(c), self.1.read_from(c))
//     }
//     fn write_to(self, c: &mut C, value: Self::FStruct) {
//         self.0.write_to(c, value.0);
//         self.1.write_to(c, value.1);
//     }

//     fn to_raw_addr(&self, c: &C) -> VecDeque<<C as Circuit>::RawAddr> {
//         let mut ret = VecDeque::new();
//         ret.append(&mut self.0.to_raw_addr(c));
//         ret.append(&mut self.1.to_raw_addr(c));
//         ret
//     }

//     fn try_from_raw_addr(c: &C, raws: &mut VecDeque<<C as Circuit>::RawAddr>) -> Self {
//         let q1 = T1::try_from_raw_addr(c, raws);
//         let q2 = T2::try_from_raw_addr(c, raws);
//         (q1, q2)
//     }
// }


// Implements for tuples.

make_tuple_impls!();


