use std::{sync::Arc, vec};
use itertools::Itertools;

use super::storage::{ReaderOf, Storage, TypedAddr, WriterOf};

pub trait RTAdvice<S: Storage> {
    fn inputs(&self) -> Vec<S::RawAddr>;
    fn outputs(&self) -> Vec<S::RawAddr>;
    fn call(&self, storage: &mut S);
}

pub struct RTGraph<S: Storage> {
    inputs: Vec<S::RawAddr>,
    outputs: Vec<S::RawAddr>,
    advices: Vec<Box<dyn RTAdvice<S>>>,
}

pub trait GraphBackend<S: Storage> {
    fn init(s: S, g: RTGraph<S>) -> Self;
    fn execute_until_input(&mut self) -> &S;
}

pub trait TypedStorageAddr<S: Storage> {
    type TypedAddr: Into<S::RawAddr>;
}

pub trait DeducteAddressesOf<DS>: Storage {
    fn addresses(ds: &DS) -> Vec<<Self as Storage>::RawAddr>;
}

pub trait AllowsStruct<DS>: Storage + DeducteAddressesOf<DS> {
    type DataSturct;
    
    fn read(&self, ds: &DS) -> Self::DataSturct;
    fn write(&mut self, ds: &DS, value: Self::DataSturct);
}


impl<S, T> DeducteAddressesOf<TypedAddr<S, T>> for S
where
    S: Storage,
{
    fn addresses(ds: &TypedAddr<S, T>) -> Vec<<Self as Storage>::RawAddr> {
        vec![S::to_raw(ds)]
    }
}

impl<S, T> AllowsStruct<TypedAddr<S, T>> for S
where
    S: Storage + ReaderOf<T> + WriterOf<T>,
    T: Clone, // This must not be here, but I am too stupid to put lifetimes in :)
{
    type DataSturct = T;

    fn read(&self, ds: &TypedAddr<S, T>) -> Self::DataSturct {
        self.get(&S::to_raw(&ds)).clone()
    }

    fn write(&mut self, ds: &TypedAddr<S, T>, value: Self::DataSturct) {
        self.put(&S::to_raw(ds.into()), value)
    }
}

impl<S, ST> DeducteAddressesOf<Vec<ST>> for S 
where 
    S: Storage + AllowsStruct<ST>
{
    fn addresses(ds: &Vec<ST>) -> Vec<<Self as Storage>::RawAddr> {
        ds.iter().map(|s| S::addresses(s)).flatten().collect()
    }
}

impl<S, ST> AllowsStruct<Vec<ST>> for S
where
    S: Storage + AllowsStruct<ST>
{
    type DataSturct = Vec<<S as AllowsStruct<ST>>::DataSturct>;

    fn read(&self, ds: &Vec<ST>) -> Self::DataSturct {
        ds.iter().map(|s| self.read(s)).collect()
    }

    fn write(&mut self, ds: &Vec<ST>, value: Self::DataSturct) {
        ds.iter().zip_eq(value.into_iter()).map(|(a, v)| self.write(a, v)).last();
    }
}

impl<S> DeducteAddressesOf<()> for S
where
    S: Storage,
{
    fn addresses(ds: &()) -> Vec<<Self as Storage>::RawAddr> {
        vec![]
    }
}

impl<S> AllowsStruct<()> for S
where
    S: Storage,
{
    type DataSturct = ();

    fn read(&self, ds: &()) -> Self::DataSturct {}

    fn write(&mut self, ds: &(), value: Self::DataSturct) {}
}

pub struct RuntimeAdvice<I, O, S>
where
    S: AllowsStruct<I>,
    S: AllowsStruct<O>,
    S: Storage,
{
    pub input: I,
    pub output: O,
    pub func: Arc<dyn Fn(<S as AllowsStruct<I>>::DataSturct) -> <S as AllowsStruct<O>>::DataSturct>,
}

impl<I, O, S> RTAdvice<S> for RuntimeAdvice<I, O, S>
where
    S: AllowsStruct<I>,
    S: AllowsStruct<O>,
    S: Storage,
{
    fn inputs(&self) -> Vec<S::RawAddr> {
        S::addresses(&self.input)
    }

    fn outputs(&self) -> Vec<S::RawAddr> {
        S::addresses(&self.output)
    }

    fn call(&self, storage: &mut S) {
        storage.write(&self.output, (*self.func)(storage.read(&self.input)));
    }
}