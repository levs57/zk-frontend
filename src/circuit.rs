use std::{marker::PhantomData};

use ff::PrimeField;
use macros::make_tuple_impls;

pub trait Circuit : Sized {
    type F : PrimeField;

    fn inputize<I: SVStruct<Self> + 'static>(s: I) -> Box<dyn Fn(&mut Self, I::FStruct)> {
        Box::new(move |circuit, value|{s.write_to(circuit, value)})
    }

    fn outputize<O: SVStruct<Self> + 'static>(s: O) -> Box<dyn Fn(&Self) -> O::FStruct> {
        Box::new(move |circuit|{s.read_from(circuit)})
    }


}

pub trait CircuitGenericAdvices : Circuit {
    fn advise<I: SVStruct<Self>, O: SVStruct<Self>, Fun: Fn(I::FStruct) -> O::FStruct>(&mut self, f: Fun, inp: I) -> O;

    fn advise_variadic<
        I: SVStruct<Self>,
        O: SVStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> O::FStruct
    > (&mut self, f: Fun, inp: &[I]) -> O;

}

pub trait CircuitGenericConstraints : Circuit {
    fn enforce<I: SigStruct<Self>, O: CRhsStruct<Self>, Fun: Fn(I::FStruct) -> O::FStruct>(&mut self, f: Fun, inp: I) -> O;

    fn enforce_variadic<
        I: SigStruct<Self>,
        O: CRhsStruct<Self>,
        Fun: Fn(&[I::FStruct]) -> Vec<O::FStruct>
    > (&mut self, f: Fun, inp: &[I]) -> O;

}

pub trait CommitmentGroup<C: Circuit> : Sized {
    type VarTarget : SVStruct<C> + 'static;

    /// Creates an empty commitment group and registers it.
    fn new(circuit: &mut C) -> Self;

    // Actually, an operation. Could be emulated by variadic advice if you are brave enough, separated for clarity.
    // This form exists in case one wants to bind it back without exiting execution graph, output form is provided later.
    #[must_use]
    fn _commit(self, circuit: &mut C) -> Self::VarTarget;

    fn commit(self, circuit: &mut C) -> Box<dyn Fn(&C) -> <Self::VarTarget as NodeStruct<C>>::FStruct> {
        C::outputize(self._commit(circuit))
    }
}

pub trait HasCGElt<SigAddr : Copy, C: Circuit> : CommitmentGroup<C> where C : HasVarAddr<Self::VarTarget>{
    fn push(&mut self, var: Self::VarTarget);
}

/// This trait allows to convert elements of type T to the ground field of the circuit.
pub trait FieldConversion<T> : Circuit {
    fn felt(value: T) -> Self::F;
}

/// Value of type T that can be held by the circuit.
pub trait HasVarAddr<VarAddr : Copy> : Circuit + Sized {
    type Value;
    fn alloc_var(&mut self) -> VarAddr;
    fn read_var(&self, var: VarAddr) -> Self::Value;
    fn write_var(&mut self, addr: VarAddr, value: Self::Value); 
}

// Section of projection Var -> Value. It seems it is impossible to bind them to be in 1-1 correspondence.
// However, it is expected. These traits should be treated more as convenience traits allowing to address
// variable type from its value type.

// pub trait HasVarValue<T> : Circuit + Sized + HasVarAddr<Self::VarAddr, Value = Self> {
//     type VarAddr : Copy;
// }

// pub trait IsVarValue<C : HasVarValue<Self>> : Sized {
//     type VarAddr;
// }
// impl<T, C : HasVarValue<T>> IsVarValue<C> for T {
//     type VarAddr = C::VarAddr;
// }


pub trait HasSigAddr<SigAddr : Copy> : Sized + FieldConversion<Self::Value> {
    type Value;
    fn alloc_sig(&mut self) -> SigAddr;
    fn read_sig(&self, sig: SigAddr) -> Self::Value;
    fn write_sig(&mut self, addr: SigAddr, value: Self::Value); 
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


pub trait HasCRhsAddr<CRhsAddr : Copy> : Sized + FieldConversion<Self::Value> {
    type Value;
    fn alloc_crhs(&mut self) -> CRhsAddr;
    fn read_crhs(&self, crhs: CRhsAddr) -> Self::Value;
    fn write_crhs(&mut self, addr: CRhsAddr, value: Self::Value); 
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

pub struct Var<VarAddr : Copy, C : HasVarAddr<VarAddr>> {
    addr: VarAddr,
    _marker: PhantomData<C>,
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Clone for Var<VarAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Copy for Var<VarAddr, C> {}


impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> From<VarAddr> for Var<VarAddr, C> {
    fn from(value: VarAddr) -> Self {
        Var {addr : value, _marker : PhantomData}
    }
}

impl <VarAddr : Copy, C : HasVarAddr<VarAddr>> Var<VarAddr, C> {
    fn addr(self) -> VarAddr {
        self.addr
    }
}

pub struct Sig<SigAddr : Copy, C : HasSigAddr<SigAddr>> {
    addr: SigAddr,
    _marker: PhantomData<C>,
}

impl <SigAddr : Copy, C : HasSigAddr<SigAddr>> Clone for Sig<SigAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <SigAddr : Copy, C : HasSigAddr<SigAddr>> Copy for Sig<SigAddr, C> {}


impl <SigAddr : Copy, C : HasSigAddr<SigAddr>> From<SigAddr> for Sig<SigAddr, C> {
    fn from(value: SigAddr) -> Self {
        Sig {addr : value, _marker : PhantomData}
    }
}

impl <SigAddr: Copy, C : HasSigAddr<SigAddr>> Sig<SigAddr, C> {
    fn addr(self) -> SigAddr {
        self.addr
    }
}
pub struct CRhs<CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> {
    addr: CRhsAddr,
    _marker: PhantomData<C>,
}

impl <CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> Clone for CRhs<CRhsAddr, C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl <CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> Copy for CRhs<CRhsAddr, C> {}


impl <CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> From<CRhsAddr> for CRhs<CRhsAddr, C> {
    fn from(value: CRhsAddr) -> Self {
        CRhs {addr : value, _marker : PhantomData}
    }
}

impl <CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> CRhs<CRhsAddr, C> {
    fn addr(self) -> CRhsAddr {
        self.addr
    }
}


/// This must represent static structures built from in-circuit runtime values.
pub trait NodeStruct<C> : Copy {
    type FStruct;

    fn alloc_to(c: &mut C) -> Self;
    fn read_from(self, c: &C) -> Self::FStruct;
    fn write_to(self, c: &mut C, value: Self::FStruct); 
}

/// This must represent static structures built from signals and variables.
pub trait SVStruct<C> : NodeStruct<C> {}

/// This must represent object built from signals.
pub trait SigStruct<C> : SVStruct<C> {}

/// This must represent object built from constraint values.
pub trait CRhsStruct<C> : NodeStruct<C> {}


// -- BASE IMPLS FOR VAR AND SIG

impl<VarAddr : Copy, C : HasVarAddr<VarAddr>> NodeStruct<C> for Var<VarAddr, C> {
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
}

impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> NodeStruct<C> for Sig<SigAddr, C> {
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
}

impl<CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> NodeStruct<C> for CRhs<CRhsAddr, C> {
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
}

impl<VarAddr : Copy, C : HasVarAddr<VarAddr>> SVStruct<C> for Var<VarAddr, C> {}
impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> SVStruct<C> for Sig<SigAddr, C> {}

impl<SigAddr : Copy, C : HasSigAddr<SigAddr>> SigStruct<C> for Sig<SigAddr, C> {}

impl<CRhsAddr : Copy, C : HasCRhsAddr<CRhsAddr>> CRhsStruct<C> for CRhs<CRhsAddr, C> {}

// -- ARRAYS

impl<C, T: NodeStruct<C>, const N: usize> NodeStruct<C> for [T; N] {
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
}

impl<C, T: SVStruct<C>, const N: usize> SVStruct<C> for [T; N] {}
impl<C, T: SigStruct<C>, const N: usize> SigStruct<C> for [T; N] {}
impl<C, T: CRhsStruct<C>, const N: usize> CRhsStruct<C> for [T; N] {}


// Implements for tuples.

make_tuple_impls!();


// pub trait CGElement<C: Circuit, T: Sigtype<C>, CG: CommitmentGroup<C>> : Signal<C, T> {
//     fn add_to(self, commitment_group: &mut CG);
// }

