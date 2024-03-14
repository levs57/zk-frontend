
use std::{any::TypeId, marker::PhantomData};

use crate::{advices::{Advices, CompileableStruct, DataStruct, TAdvice}, backend::{api::AllowsStruct, storage::{AllocatorOf, ReaderOf, Storage, WriterOf}}, circuit::{Advice, Circuit, Conversion, HasSigtype, HasVartype, PrimarySignalFlag, SignalFlag, Signals, VariableFlag}, gadgets::traits::atoms::Pow5};


type F = halo2curves::bn256::Fr;

struct ExampleStorage {
    fs: Vec<Option<F>>,
    u64s: Vec<Option<u64>>,
}

#[derive(Clone, Copy)]
struct ExampleRawAddr {
    addr: usize,
    typeid: TypeId,
}

impl Storage for ExampleStorage {
    type RawAddr = ExampleRawAddr;

    fn to_raw<T>(ta: &crate::backend::storage::TypedAddr<Self, T>) -> Self::RawAddr where Self: Sized {
        ta.addr
    }
}

impl AllocatorOf<F> for ExampleStorage {
    fn allocate(&mut self) -> <Self as Storage>::RawAddr {
        self.fs.push(None);
        ExampleRawAddr {
            addr: self.fs.len() - 1,
            typeid: TypeId::of::<F>(),
        }
    }
}

impl ReaderOf<F> for ExampleStorage {
    fn get(&self, addr: &Self::RawAddr) -> &F {
        self.fs.get(addr.addr).unwrap().as_ref().unwrap()
    }
}

impl WriterOf<F> for ExampleStorage {
    fn put(&mut self, addr: &Self::RawAddr, val: F) {
        self.fs[addr.addr] = Some(val)
    }
}

impl AllocatorOf<u64> for ExampleStorage {
    fn allocate(&mut self) -> <Self as Storage>::RawAddr {
        self.u64s.push(None);
        ExampleRawAddr {
            addr: self.u64s.len() - 1,
            typeid: TypeId::of::<F>(),
        }
    }
}

impl ReaderOf<u64> for ExampleStorage {
    fn get(&self, addr: &Self::RawAddr) -> &u64 {
        self.u64s.get(addr.addr).unwrap().as_ref().unwrap()
    }
}

impl WriterOf<u64> for ExampleStorage {
    fn put(&mut self, addr: &Self::RawAddr, val: u64) {
        self.u64s[addr.addr] = Some(val)
    }
}


struct ExampleCircuit {
    allocation_addr: usize,
    advices: Vec<Box<dyn TAdvice<Self, ExampleStorage, dyn Fn(<ExampleCircuit as Circuit>::RawAddr)>>>,
}

impl Conversion<halo2curves::bn256::Fr, halo2curves::bn256::Fr> for ExampleCircuit {
    fn convert(value: halo2curves::bn256::Fr) -> halo2curves::bn256::Fr {
        value
    }
}

impl HasVartype<halo2curves::bn256::Fr> for ExampleCircuit {}
impl HasSigtype<halo2curves::bn256::Fr> for ExampleCircuit {}


impl Circuit for ExampleCircuit {
    type F = halo2curves::bn256::Fr;

    type RawAddr = usize;

    type Config = Self;

    fn inner_type(&self, addr: Self::RawAddr) -> std::any::TypeId {
        unreachable!()
    }

    fn _alloc_raw<T: 'static>(&mut self) -> Self::RawAddr where Self::Config : crate::circuit::HasVartype<T> {
        let ret = self.allocation_addr;
        self.allocation_addr += 1;
        ret
    }
}

impl VariableFlag for ExampleCircuit {
    fn is_var(&self, addr: Self::RawAddr) -> bool {
        unreachable!()
    }

    fn _set_var_flag(&mut self, addr: Self::RawAddr, value: bool) {
        unreachable!()
    }
}

impl SignalFlag for ExampleCircuit {
    fn is_sig(&self, addr: Self::RawAddr) -> bool {
        unreachable!()
    }

    fn _set_sig_flag(&mut self, addr: Self::RawAddr, value: bool) {
        unreachable!()
    }
}

impl PrimarySignalFlag for ExampleCircuit {
    fn is_primary(&self, addr: Self::RawAddr) -> bool {
        unreachable!()
    }

    fn _set_primary_flag(&mut self, addr: Self::RawAddr, value: bool) {
        unreachable!()
    }
}

impl Advices for ExampleCircuit {
    type Storage = ExampleStorage;

    fn advise_to_unassigned<I, CI, O, CO, F: Fn(I::FStruct) -> O::FStruct>(&mut self, f: F, input: I, output: O)
    where 
        I: DataStruct<Self>,
        Self::Storage: AllowsStruct<CI, DataSturct = <I as DataStruct<Self>>::FStruct>,
        I: CompileableStruct<Self, Self::Storage, Box<dyn Fn(<Self as Circuit>::RawAddr) -> <<Self as Advices>::Storage as Storage>::RawAddr>, CI>,
        
        O: DataStruct<Self>,
        Self::Storage: AllowsStruct<CO, DataSturct = <O as DataStruct<Self>>::FStruct>,
        O: CompileableStruct<Self, Self::Storage, Box<dyn Fn(<Self as Circuit>::RawAddr) -> <<Self as Advices>::Storage as Storage>::RawAddr>, CO>,
    {
        self.advices.push(Box::new(
            Advice {
                input,
                output,
                func: f,
                _pd: PhantomData,
            }
        ))
    }

    fn compile_execution_graph(&self) -> crate::backend::api::RTGraph<Self::Storage> {
        todo!()
    }
}