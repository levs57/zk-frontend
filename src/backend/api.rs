use super::storage::Storage;

pub trait RTAdvice {
    type Storage: Storage;
    fn inputs(&self) -> Vec<<Self::Storage as Storage>::RawAddr>;
    fn outputs(&self) -> Vec<<Self::Storage as Storage>::RawAddr>;
    fn call(&self, storage: &mut Self::Storage);
}

pub struct RTGraph<S: Storage> {
    inputs: Vec<S::RawAddr>,
    outputs: Vec<S::RawAddr>,
    advices: Vec<Box<dyn RTAdvice<Storage = S>>>,
}

pub trait GraphBackend<S: Storage> {
    fn init(s: S, g: RTGraph<S>) -> Self;
    fn execute_until_input(&mut self) -> &S;
}