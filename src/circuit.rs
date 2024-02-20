use std::{any::TypeId, marker::PhantomData};

use ff::PrimeField;
use num_bigint::BigUint;



pub trait FieldUtils {

}

/// Bootleg Into (to avoid conflicting implementations).
pub trait _Into<T : ?Sized> {
    fn _into(self) -> T;
}

pub trait ToRawAddr<C: Circuit> {
    fn to_raw_addr(&self) -> C::RawAddr;
}

/// This could be Into trait, but orphan rule otherwise.
pub trait Conversion<A, B> {
    fn convert(value: A) -> B;
}

impl<C : Circuit> Conversion<u64, C::F> for C {
    #[inline(always)]
    fn convert(value: u64) -> C::F {
        value.into()
    }
}

impl<C : Circuit> Conversion<u32, C::F> for C {
    #[inline(always)]
    fn convert(value: u32) -> C::F {
        <Self as Conversion<u64, _>>::convert(value as u64)
    }
}

impl<C : Circuit> Conversion<u16, C::F> for C {
    #[inline(always)]
    fn convert(value: u16) -> C::F {
        <Self as Conversion<u64, _>>::convert(value as u64)
    }
}

impl<C : Circuit> Conversion<u8, C::F> for C {
    #[inline(always)]
    fn convert(value: u8) -> C::F {
        <Self as Conversion<u64, _>>::convert(value as u64)
    }
}

impl<C : Circuit> Conversion<bool, C::F> for C {
    #[inline(always)]
    fn convert(value: bool) -> C::F {
        <Self as Conversion<u64, _>>::convert(value as u64)
    }
}




pub trait HasVartype<T: 'static> : Circuit + Conversion<T, T> {}
pub trait HasSigtype<T: 'static> : Circuit + HasVartype<T> + Conversion<T, Self::F> {}



// --------------------------------------------
// Circuit flags
pub trait Circuit : Conversion<Self::F, Self::F> + Sized{
    type F : PrimeField;
    type RawAddr : Copy;

    fn inner_type(&self, addr: Self::RawAddr) -> TypeId;
    /// Constructs a new raw address with inner type T. All boolean flags are unset, all other flags are None.
    fn _alloc_raw<T: 'static>(&mut self) -> Self::RawAddr where Self : HasVartype<T>;
}

pub trait VariableFlag : Circuit {
    /// Checks whether raw address contains a variable (i.e. r/w node at graph execution runtime).
    fn is_var(&self, addr: Self::RawAddr) -> bool;
    // Unsafe. Sets variable flag.
    fn _set_var_flag(&mut self, addr: Self::RawAddr, value: bool);
}

pub trait SignalFlag : Circuit + VariableFlag {
    /// Checks whether raw address contains a signal (i.e. constrainable value).
    fn is_sig(&self, addr: Self::RawAddr) -> bool;
    /// Unsafe. Sets signal flag.
    fn _set_sig_flag(&mut self, addr: Self::RawAddr, value: bool);
}
pub trait CommittedFlag : Circuit {
    /// Checks whether a raw address contains a committed signal (some signals are linear combinations of others).
    fn is_committed(&self, addr: Self::RawAddr) -> bool;
    /// Unsafe. Sets signal flag.
    fn _set_committed_flag(&mut self, addr: Self::RawAddr, value: bool);
}

pub trait RangeBound : Circuit {
    /// Checks the range bound of a signal. None if no range bound is known.
    fn bound(&self, addr: Self::RawAddr) -> Option<BigUint>;
    /// Unsafe. Sets the range bound of a signal, ignoring already existing bounds.
    fn _set_bound(&mut self, addr: Self::RawAddr, value: Option<&BigUint>);
}
pub trait ConstrRhsFlag : Circuit {
    /// Checks whether value is constraint rhs.
    fn is_constr_rhs(&self, addr: Self::RawAddr) -> bool;
    /// Unsafe. Sets value to be constraint rhs.
    fn _set_constr_rhs_flag(&mut self, addr: Self::RawAddr, value: bool);
}

pub trait ConstantFlag : Circuit + VariableFlag {
    /// Checks whether value is constant.
    fn is_const(&self, addr: Self::RawAddr) -> bool;
    /// Unsafe. Sets value to be constant.
    fn _set_const_flag(&mut self, addr: Self::RawAddr, value: bool);
}

// --------------------------------------------
// Typed wrappers

// ---------VARS---------


pub trait Variables : Circuit + VariableFlag {
    type Var<T> : Copy + ToRawAddr<Self> where T: 'static, Self : HasVartype<T>;
    fn var_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Var<T> where Self : HasVartype<T>;
    fn alloc_var<T: 'static>(&mut self) -> Self::Var<T> where Self : HasVartype<T>;
}

impl<C : Circuit + VariableFlag> Variables for C {
    type Var<T : 'static> = Var<C, T> where C : HasVartype<T>;

    fn var_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Var<T> where C : HasVartype<T>{
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        Var {raw_addr, _marker : PhantomData}
    }

    fn alloc_var<T: 'static>(&mut self) -> Self::Var<T> where Self : HasVartype<T>, {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self.var_from_raw_addr(raw_addr)
    }
}

/// Anything that deals with advices is a struct of vars (and, generally, any node value in execution graph).
/// Signals and other things can be converted to it using _into().
pub struct Var<C: Circuit + HasVartype<T>, T: 'static> {
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C: Circuit + HasVartype<T>, T: 'static> Clone for Var<C, T> {
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C: Circuit + VariableFlag + HasVartype<T>, T: 'static> Copy for Var<C, T> {}

impl<C: Circuit + VariableFlag + HasVartype<T1> + HasVartype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Var<C, T1> 
{
    /// Converts the variable with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C: Circuit + HasVartype<T>, T: 'static> ToRawAddr<C> for Var<C, T> {
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}


// ---------SIGS---------

pub trait Signals : Circuit + SignalFlag + CommittedFlag {
    type Sig<T> : Copy + ToRawAddr<Self> where T: 'static, Self : HasSigtype<T>;
    fn sig_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Sig<T> where Self : HasSigtype<T>;

    fn alloc_sig_uncommitted<T: 'static>(&mut self) -> Self::Sig<T> where Self : HasSigtype<T>;
    fn alloc_sig_committed<T: 'static>(&mut self) -> Self::Sig<T> where Self : HasSigtype<T>;
}

impl<C : Circuit + SignalFlag + CommittedFlag> Signals for C {
    type Sig<T : 'static> = Sig<C, T> where C : HasSigtype<T>;

    fn sig_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Sig<T> where Self : HasSigtype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        assert!(self.is_sig(raw_addr));
        Sig {raw_addr, _marker : PhantomData}
    }

    fn alloc_sig_uncommitted<T: 'static>(&mut self) -> Self::Sig<T> where Self : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self._set_sig_flag(raw_addr, true);
        self.sig_from_raw_addr(raw_addr)
    }

    fn alloc_sig_committed<T: 'static>(&mut self) -> Self::Sig<T> where Self : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self._set_sig_flag(raw_addr, true);
        self._set_committed_flag(raw_addr, true);
        self.sig_from_raw_addr(raw_addr)
    }
}


pub struct Sig<C: Circuit + HasSigtype<T>, T: 'static> {
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C: Circuit + HasSigtype<T>, T: 'static> Clone for Sig<C, T> {
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C: Circuit + HasSigtype<T>, T: 'static> Copy for Sig<C, T> {}

impl<C: Circuit + HasSigtype<T1> + HasSigtype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Sig<C, T2>> for Sig<C, T1>
{
    /// Converts the signal with inner type T1 into signal with outer type T2.
    #[inline(always)]
    fn _into(self) -> Sig<C, T2> {
        Sig { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C: Circuit + HasSigtype<T1> + HasSigtype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Sig<C, T1>
{
    /// Converts the signal with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C: Circuit + HasSigtype<T>, T: 'static> ToRawAddr<C> for Sig<C, T> {
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

// ---------CONSTS---------

pub trait Constants : Circuit + ConstantFlag {
    type Const<T> : Copy where T: 'static, Self : HasVartype<T>;
    fn const_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Const<T> where Self : HasVartype<T>;
    fn alloc_const<T: 'static>(&mut self) -> Self::Const<T> where Self : HasVartype<T>;
}

impl<C : Circuit + ConstantFlag> Constants for C {
    type Const<T> = Const<C, T> where T: 'static, Self : HasVartype<T>;

    fn const_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::Const<T> where Self : HasVartype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        assert!(self.is_const(raw_addr));
        Const {raw_addr, _marker : PhantomData}
    }

    fn alloc_const<T: 'static>(&mut self) -> Self::Const<T> where Self : HasVartype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_const_flag(raw_addr, true);
        self._set_var_flag(raw_addr, true);
        self.const_from_raw_addr(raw_addr)
    }
}

pub struct Const<C: Circuit + HasVartype<T>, T: 'static> {
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C: Circuit + HasVartype<T>, T: 'static> Clone for Const<C, T> {
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C: Circuit + HasVartype<T>, T: 'static> Copy for Const<C, T> {}

impl<C: Circuit + HasVartype<T1> + HasVartype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Const<C, T2>> for Const<C, T1>
{
    /// Converts the const with inner type T1 into const with outer type T2.
    #[inline(always)]
    fn _into(self) -> Const<C, T2> {
        Const { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C: Circuit + HasVartype<T1> + HasVartype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Const<C, T1>
{
    /// Converts the const with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}


// ---------CONSTR RHS---------

pub trait ConstrRhss : Circuit + ConstrRhsFlag {
    type ConstrRhs<T> : Copy where T: 'static, Self : HasSigtype<T>;
    fn constr_rhs_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::ConstrRhs<T> where Self : HasSigtype<T>;
    fn alloc_constr_rhs<T: 'static>(&mut self) -> Self::ConstrRhs<T> where Self : HasSigtype<T>;
}

impl<C : Circuit + ConstrRhsFlag> ConstrRhss for C {
    type ConstrRhs<T> = ConstrRhs<C, T> where T: 'static, Self : HasSigtype<T>;

    fn constr_rhs_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Self::ConstrRhs<T> where Self : HasSigtype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_constr_rhs(raw_addr));
        ConstrRhs {raw_addr, _marker : PhantomData}
    }

    fn alloc_constr_rhs<T: 'static>(&mut self) -> Self::ConstrRhs<T> where Self : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_constr_rhs_flag(raw_addr, true);
        self.constr_rhs_from_raw_addr(raw_addr)
    }
}

pub struct ConstrRhs<C: Circuit + HasSigtype<T>, T: 'static> {
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C: Circuit + HasSigtype<T>, T: 'static> Clone for ConstrRhs<C, T> {
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C: Circuit + HasSigtype<T>, T: 'static> Copy for ConstrRhs<C, T> {}

// --------- Access permissions -----------
// --------- Untyped permission markers over raw_addr ------------

#[derive(Clone, Copy)]
pub struct RWPermit<C: Circuit> {
    raw_addr : C::RawAddr,
}

impl<C: Circuit> RWPermit<C> {

}


// --------- RWSTRUCT --------

pub trait RWStruct<C : Circuit> {
    type FStruct;
    
}

// --------- ADVICES ---------

pub trait Advices : Circuit {
    type Advice<I, O>;
}



pub trait _From<T : _Into<Self> + ?Sized> {
    fn _from(value: T) -> Self;
}

impl<T1: _Into<T2>, T2> _From<T1> for T2 {
    fn _from(value: T1) -> Self {
        value._into()
    }
}

