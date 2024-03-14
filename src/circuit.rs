use std::{any::TypeId, marker::PhantomData, sync::Arc};

use ff::PrimeField;
use num_bigint::BigUint;

use crate::{advices::{CompileableStruct, DataStruct, TAdvice}, backend::{api::{AllowsStruct, RuntimeAdvice, RTAdvice}, storage::{Storage, TypedAddr}}};


pub trait FieldUtils {

}

/// Bootleg Into (to avoid conflicting implementations).
pub trait _Into<T : ?Sized> {
    fn _into(self) -> T;
}

pub trait _From<T : _Into<Self> + ?Sized> {
    fn _from(value: T) -> Self;
}

impl<T1: _Into<T2>, T2> _From<T1> for T2 {
    fn _from(value: T1) -> Self {
        value._into()
    }
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
    type Config;

    fn inner_type(&self, addr: Self::RawAddr) -> TypeId;
    /// Constructs a new raw address with inner type T. All boolean flags are unset, all other flags are None.
    fn _alloc_raw<T: 'static>(&mut self) -> Self::RawAddr where Self::Config : HasVartype<T>;
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
pub trait PrimarySignalFlag : Circuit + SignalFlag{
    /// Checks whether a raw address contains a primary signal.
    fn is_primary(&self, addr: Self::RawAddr) -> bool;
    /// Unsafe. Sets primary flag.
    fn _set_primary_flag(&mut self, addr: Self::RawAddr, value: bool);
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
    fn var_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Var<Self, T> where Self::Config : HasVartype<T>;
    fn alloc_var<T: 'static>(&mut self) -> Var<Self, T> where Self::Config : HasVartype<T>;
}

impl<C : Circuit + VariableFlag> Variables for C {
    fn var_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Var<Self, T> where C::Config : HasVartype<T>{
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        Var {raw_addr, _marker : PhantomData}
    }

    fn alloc_var<T: 'static>(&mut self) -> Var<Self, T> where Self::Config : HasVartype<T>, {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self.var_from_raw_addr(raw_addr)
    }
}

/// Anything that deals with advices is a struct of vars (and, generally, any node value in execution graph).
/// Signals and other things can be converted to it using _into().
pub struct Var<C, T: 'static>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C, T: 'static> Clone for Var<C, T> 
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C, T: 'static> Copy for Var<C, T>
where
    C: Circuit + VariableFlag,
    C::Config: HasVartype<T>,
{}

impl<C: Circuit + VariableFlag + HasVartype<T1> + HasVartype<T2> + Conversion<T1, T2>, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Var<C, T1> 
where
    C: Circuit + VariableFlag + Conversion<T1, T2>,
    C::Config: HasVartype<T1> + HasVartype<T2>,
{
    /// Converts the variable with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C, T: 'static> ToRawAddr<C> for Var<C, T>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}


// ---------SIGS---------

pub trait Signals : Circuit + SignalFlag + PrimarySignalFlag {
    fn sig_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Sig<Self, T> where Self::Config : HasSigtype<T>;
    /// Allocates signal and commits it.
    fn alloc_sig<T: 'static>(&mut self) -> Sig<Self, T> where  Self::Config : HasSigtype<T>;
    /// Allocates signal and does not commit it. Unsafe. Should be only used in conjunction with linear combination constraint.
    fn _alloc_sig_dependent<T: 'static>(&mut self) -> Sig<Self, T> where  Self::Config : HasSigtype<T>;
}

impl<C : Circuit + SignalFlag + PrimarySignalFlag> Signals for C {
    fn sig_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Sig<Self, T> where  Self::Config : HasSigtype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        assert!(self.is_sig(raw_addr));
        Sig {raw_addr, _marker : PhantomData}
    }

    fn alloc_sig<T: 'static>(&mut self) -> Sig<Self, T> where Self::Config : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self._set_sig_flag(raw_addr, true);
        self._set_primary_flag(raw_addr, true);
        self.sig_from_raw_addr(raw_addr)
    }

    fn _alloc_sig_dependent<T: 'static>(&mut self) -> Sig<Self, T> where Self::Config : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_var_flag(raw_addr, true);
        self._set_sig_flag(raw_addr, true);
        self.sig_from_raw_addr(raw_addr)
    }
}


pub struct Sig<C: Circuit, T: 'static> 
where 
    C::Config: HasSigtype<T>
{
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}

impl<C: Circuit, T: 'static> Clone for Sig<C, T>
where
    C::Config: HasSigtype<T>    
{
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C: Circuit, T: 'static> Copy for Sig<C, T> 
where
    C::Config: HasSigtype<T>    
{}

impl<C, T1: 'static, T2: 'static>
    _Into<Sig<C, T2>> for Sig<C, T1>
where 
    C: Circuit + Conversion<T1, T2>,
    C::Config: HasSigtype<T1> + HasSigtype<T2>,
{
    /// Converts the signal with inner type T1 into signal with outer type T2.
    #[inline(always)]
    fn _into(self) -> Sig<C, T2> {
        Sig { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Sig<C, T1>
where
    C: Circuit + Conversion<T1, T2>,
    C::Config: HasSigtype<T1> + HasSigtype<T2>,
{
    /// Converts the signal with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C, T: 'static> ToRawAddr<C> for Sig<C, T> 
where
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

// ---------CONSTS---------

pub trait Constants : Circuit + ConstantFlag {
    fn const_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Const<Self, T> where Self::Config : HasVartype<T>;
    fn alloc_const<T: 'static>(&mut self) -> Const<Self, T> where Self::Config : HasVartype<T>;
}

impl<C : Circuit + ConstantFlag> Constants for C {
    fn const_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> Const<Self, T> where Self::Config : HasVartype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_var(raw_addr));
        assert!(self.is_const(raw_addr));
        Const {raw_addr, _marker : PhantomData}
    }

    fn alloc_const<T: 'static>(&mut self) -> Const<Self, T> where Self::Config : HasVartype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_const_flag(raw_addr, true);
        self._set_var_flag(raw_addr, true);
        self.const_from_raw_addr(raw_addr)
    }
}

pub struct Const<C, T: 'static>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}

impl<C, T: 'static> Clone for Const<C, T>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}

impl<C, T: 'static> Copy for Const<C, T>
where
    C: Circuit,
    C::Config: HasVartype<T>
{}

impl<C, T1: 'static, T2: 'static>
    _Into<Const<C, T2>> for Const<C, T1>
where
    C: Circuit + Conversion<T1, T2>,
    C::Config: HasVartype<T1> + HasVartype<T2>,
{
    /// Converts the const with inner type T1 into const with outer type T2.
    #[inline(always)]
    fn _into(self) -> Const<C, T2> {
        Const { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C, T1: 'static, T2: 'static>
    _Into<Var<C, T2>> for Const<C, T1>
where
    C: Circuit + Conversion<T1, T2>,
    C::Config: HasVartype<T1> + HasVartype<T2>,
{
    /// Converts the const with inner type T1 into variable with outer type T2.
    #[inline(always)]
    fn _into(self) -> Var<C, T2> {
        Var { raw_addr : self.raw_addr, _marker : PhantomData }
    }
}

impl<C, T: 'static> ToRawAddr<C> for Const<C, T>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

// ---------CONSTR RHS---------

pub trait ConstrRhss : Circuit + ConstrRhsFlag {
    fn constr_rhs_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> ConstrRhs<Self, T> where Self::Config : HasSigtype<T>;
    fn alloc_constr_rhs<T: 'static>(&mut self) -> ConstrRhs<Self, T> where Self::Config : HasSigtype<T>;
}

impl<C : Circuit + ConstrRhsFlag> ConstrRhss for C {
    fn constr_rhs_from_raw_addr<T: 'static>(&self, raw_addr: Self::RawAddr) -> ConstrRhs<Self, T> where Self::Config : HasSigtype<T> {
        assert!(self.inner_type(raw_addr) == TypeId::of::<T>());
        assert!(self.is_constr_rhs(raw_addr));
        ConstrRhs {raw_addr, _marker : PhantomData}
    }

    fn alloc_constr_rhs<T: 'static>(&mut self) -> ConstrRhs<Self, T> where Self::Config : HasSigtype<T> {
        let raw_addr = self._alloc_raw::<T>();
        self._set_constr_rhs_flag(raw_addr, true);
        self.constr_rhs_from_raw_addr(raw_addr)
    }
}

pub struct ConstrRhs<C, T: 'static>
where
    C: Circuit,
    C::Config: HasSigtype<T>
{
    raw_addr : C::RawAddr,
    _marker : PhantomData<T>,
}
impl<C:, T: 'static> Clone for ConstrRhs<C, T>
where
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    fn clone(&self) -> Self {
        Self { raw_addr: self.raw_addr.clone(), _marker: PhantomData }
    }
}
impl<C, T: 'static> Copy for ConstrRhs<C, T>
where
    C: Circuit,
    C::Config:HasSigtype<T>,
{}

impl<C, T: 'static> ToRawAddr<C> for ConstrRhs<C, T>
where
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

// --------- Access permissions -----------
// --------- Untyped permission markers over raw_addr ------------

#[derive(Clone, Copy)]
pub struct RWPermit<C: Circuit> {
    raw_addr : C::RawAddr,
}

impl<C: Circuit> ToRawAddr<C> for RWPermit<C> {
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

impl<C, T: 'static> _Into<RWPermit<C>> for Var<C, T> 
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn _into(self) -> RWPermit<C> {
        RWPermit { raw_addr: self.raw_addr }
    }
}

impl<C, T: 'static> _Into<RWPermit<C>> for Sig<C, T>
where
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    fn _into(self) -> RWPermit<C> {
        RWPermit { raw_addr: self.raw_addr }
    }
}

impl<C: Circuit + HasVartype<T>, T: 'static> _Into<RWPermit<C>> for Const<C, T>
where
    C: Circuit,
    C::Config: HasVartype<T>,
{
    fn _into(self) -> RWPermit<C> {
        RWPermit { raw_addr: self.raw_addr }
    }
}

#[derive(Clone, Copy)]
pub struct CsPermit<C: Circuit> {
    raw_addr : C::RawAddr,
}

impl<C: Circuit> ToRawAddr<C> for CsPermit<C> {
    fn to_raw_addr(&self) -> <C as Circuit>::RawAddr {
        self.raw_addr
    }
}

impl<C, T: 'static> _Into<CsPermit<C>> for Sig<C, T>
where
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    fn _into(self) -> CsPermit<C> {
        CsPermit { raw_addr: self.raw_addr }
    }
}


// --------- RWSTRUCT --------


// --------- CSSTRUCT --------

pub trait CsStruct<C: Circuit> :  DataStruct<C>{
    fn serialize_cs(&self) -> Vec<CsPermit<C>>;
}


// --------- ADVICES ---------


impl<T, C> DataStruct<C> for Sig<C, T>
where 
    C: Circuit,
    C::Config: HasSigtype<T>,
{
    type FStruct = C::F;

    fn alloc_to(c: &mut C) -> Self {
        todo!()
    }
}

impl<C, S, F, T> CompileableStruct<C, S, F, TypedAddr<S, T>> for Sig<C, T>
where
    C: Circuit,
    C:: Config: HasSigtype<T>,
    S: Storage + AllowsStruct<TypedAddr<S, T>>,
    F: Fn(C::RawAddr) -> S::RawAddr,
{
    fn compile(&self, s: &mut S, mapping: F) -> TypedAddr<S, T> {
        TypedAddr {
            addr: mapping(self.raw_addr),
            _pd: PhantomData,
        }
    }
}

pub struct Advice<I, DI, O, DO, C, S, F>
where
    I: DataStruct<C>,
    I: CompileableStruct<C, S, F, DI>,
    S: AllowsStruct<DI>,
    O: DataStruct<C>,
    O: CompileableStruct<C, S, F, DO>,
    S: AllowsStruct<DO>,
    C: Circuit,
    S: Storage,
    F: Fn(C::RawAddr) -> S::RawAddr,
{
    pub input: I,
    pub output: O,
    pub func: Arc<dyn Fn(<S as AllowsStruct<DI>>::DataSturct) -> <S as AllowsStruct<DO>>::DataSturct>,
    pub _pd: PhantomData<(C, F)>,
}

impl<I, DI, O, DO, C, S, F> TAdvice<C, S, F> for Advice<I, DI, O, DO, C, S, F>
where
    DI: 'static,
    DO: 'static,
    S: 'static,
    I: DataStruct<C>,
    I: CompileableStruct<C, S, F, DI>,
    S: AllowsStruct<DI>,
    O: DataStruct<C>,
    O: CompileableStruct<C, S, F, DO>,
    S: AllowsStruct<DO>,
    C: Circuit,
    S: Storage,
    F: Clone + Fn(C::RawAddr) -> S::RawAddr,
{
    fn compile(&self, s: &mut S, mapping: F) -> Box<dyn RTAdvice<S>> {
        Box::new(RuntimeAdvice {
            input: self.input.compile(s, mapping.clone()),
            output: self.output.compile(s, mapping),
            func: self.func.clone(),
        })
    }
}

impl<I, DI, O, DO, C, S, F> TAdvice<C, S, F> for Box<Advice<I, DI, O, DO, C, S, F>>
where
    DI: 'static,
    DO: 'static,
    S: 'static,
    I: DataStruct<C>,
    I: CompileableStruct<C, S, F, DI>,
    S: AllowsStruct<DI>,
    O: DataStruct<C>,
    O: CompileableStruct<C, S, F, DO>,
    S: AllowsStruct<DO>,
    C: Circuit,
    S: Storage,
    F: Clone + Fn(C::RawAddr) -> S::RawAddr,
{
    fn compile(&self, s: &mut S, mapping: F) -> Box<dyn RTAdvice<S>> {
        Box::new(RuntimeAdvice {
            input: self.input.compile(s, mapping.clone()),
            output: self.output.compile(s, mapping),
            func: self.func.clone(),
        })
    }
}
