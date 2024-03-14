use crate::backend::{api::{AllowsStruct, RTAdvice, RTGraph}, storage::Storage};

use crate::circuit::Circuit;

pub trait DataStruct<C: Circuit> {
    type FStruct;
    
    fn alloc_to(c: &mut C) -> Self;
}

pub trait CompileableStruct<C, S, F, T>: DataStruct<C>
where
    C: Circuit,
    S: Storage,
    F: Fn(C::RawAddr) -> S::RawAddr,
    S: AllowsStruct<T>,
{
    fn compile(&self, s: &mut S, mapping: F) -> T;
}

impl<C: Circuit> DataStruct<C> for () {
    type FStruct = ();

    fn alloc_to(c: &mut C) -> Self {}
}

impl<C, S, F> CompileableStruct<C, S, F, ()> for ()
where
    C: Circuit,
    S: Storage,
    F: Fn(C::RawAddr) -> S::RawAddr,
{
    fn compile(&self, s: &mut S, mapping: F) -> () {}
}

pub trait Advices : Circuit {
    type Storage: Storage;
    fn advise_to_unassigned<I, CI, O, CO, F: Fn(I::FStruct) -> O::FStruct>(&mut self, f: F, input: I, output: O)
    where
        I: DataStruct<Self>,
        Self::Storage: AllowsStruct<CI, DataSturct = <I as DataStruct<Self>>::FStruct>,
        I: CompileableStruct<Self, Self::Storage, Box<dyn Fn(<Self as Circuit>::RawAddr) -> <<Self as Advices>::Storage as Storage>::RawAddr>, CI>,
        
        O: DataStruct<Self>,
        Self::Storage: AllowsStruct<CO, DataSturct = <O as DataStruct<Self>>::FStruct>,
        O: CompileableStruct<Self, Self::Storage, Box<dyn Fn(<Self as Circuit>::RawAddr) -> <<Self as Advices>::Storage as Storage>::RawAddr>, CO>,
    ;

    // fn advise<I: DataStruct<Self>, O: DataStruct<Self>, F: Fn(I::FStruct) -> O::FStruct>(&mut self, f: F, input: I) -> O {
    //     let output = O::alloc_to(self);
    //     self.advise_to_unassigned(f, input, output);
    //     output
    // }

    fn compile_execution_graph(&self) -> RTGraph<Self::Storage>;
}

pub trait TAdvice<C, S, F>
where
    C: Circuit,
    S: Storage,
    F: Fn(C::RawAddr) -> S::RawAddr,
{
    fn compile(&self, s: &mut S, mapping: F) -> Box<dyn RTAdvice<S>>;
}

