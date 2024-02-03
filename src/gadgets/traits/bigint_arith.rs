use num_bigint::BigUint;

use crate::circuit::{Circuit, Sig, StandardVariables};

pub trait RangecheckImpl<C: Circuit + StandardVariables> {
    /// A version of a signal that is bounded from above on construction phase.
    /// Might be represented as non-canonical signal address internally (for more
    /// efficient commitment), but considering it must convert to Sig and from Sig
    /// implementor then MUST ensure that Sig is a proxy address capable of pointing
    /// to both normal and bounded addresses.
    type BoundedSig : Copy;

    /// Returns upper bound of this signal.
    fn bound(c: & C, sig: Self::BoundedSig) -> BigUint;

    /// Increases bound to a new one. MUST fail if the suggested bound is smaller.
    /// Note that while sig is immutable, it is an address, which points to the actual bound data.
    fn promote(c: &mut C, sig: Self::BoundedSig, new_bound: &BigUint);

    /// Converts the signal into the bounded one, assuming the bound.
    /// Ideally, should not require a copy constraint.
    /// Should return None if the bound is larger than the field modulus.
    fn assume_bounded(c: &mut C, sig: Sig<C>, bound: &BigUint) -> Option<Self::BoundedSig>;

    /// Returns normal signal representation.
    /// Ideally, should not require a copy constraint.
    fn as_sig(c: &C, sig: Self::BoundedSig) -> Sig<C>;

    /// Returns non-negative linear combination. Panics in case of bound overflow.
    fn num_linear_combination(c: &mut C, coeffs: &[BigUint], values: &[Self::BoundedSig]) -> Self::BoundedSig;
    
    /// Returns the product. Panics in case of bound overflow.
    fn num_mul(c: &mut C, a: Self::BoundedSig, b: Self::BoundedSig) -> Self::BoundedSig;

    /// Returns the maximal bound that is achievable in a single check.
    /// Check of sizes lesser than this MIGHT be implemented as two checks on x and x-k.
    fn max_primitive_rangecheck(c: &C) -> usize;

    /// Range-checks the signal. Fails if bound > max_primitive_rangecheck
    fn primitive_rangecheck(c: &mut C, sig: Sig<C>, bound: &BigUint) -> Self::BoundedSig;

    /// Splits the value into limbs with some base. Does not constrain anything.
    fn advise_split_into_limbs(c: &mut C, sig: Sig<C>, base: &BigUint, num_limbs: u32) -> Vec<Sig<C>>;

    /// Splits the signal into limbs. Returns Option<range-checked signal> and a vector of range-checked limbs.
    /// Should fail if base > max_primitive_rangecheck.
    /// In case where base.pow(num_limbs) > field modulus, None is returned in place of range-checked signal,
    /// and then the uniqueness of decomposition is not guaranteed. 
    fn split_into_limbs_nonunique(c: &mut C, sig: Sig<C>, base: &BigUint, num_limbs: u32) -> (Vec<Self::BoundedSig>, Option<Self::BoundedSig>) {
        let sigs = Self::advise_split_into_limbs(c, sig, base, num_limbs);
        let sigs = sigs.into_iter().map(|sig| Self::primitive_rangecheck(c, sig, base)).collect();
        let sig = Self::assume_bounded(c, sig, &base.pow(num_limbs));
        (sigs, sig)
    }
}

pub trait Rangecheck<ImplInstance: RangecheckImpl<Self>> : Circuit + StandardVariables {
    /// Returns upper bound of this signal.
    fn bound(&self, sig: ImplInstance::BoundedSig) -> BigUint {
        ImplInstance::bound(self, sig)
    }

    /// Increases bound to a new one. MUST fail if the suggested bound is smaller.
    /// Note that while sig is immutable, it is an address, which points to the actual bound data.
    fn promote(&mut self, sig: ImplInstance::BoundedSig, new_bound: &BigUint) {
        ImplInstance::promote(self, sig, new_bound)
    }

    /// Converts the signal into the bounded one, assuming the bound.
    /// Ideally, should not require a copy constraint.
    fn assume_bounded(&mut self, sig: Sig<Self>, bound: &BigUint) -> Option<ImplInstance::BoundedSig> {
        ImplInstance::assume_bounded(self, sig, bound)
    }

    /// Returns normal signal representation.
    /// Ideally, should not require a copy constraint.
    fn as_sig(&self, sig: ImplInstance::BoundedSig) -> Sig<Self> {
        ImplInstance::as_sig(self, sig)
    }

    /// Returns non-negative linear combination. Panics in case of bound overflow.
    fn num_linear_combination(&mut self, coeffs: &[BigUint], values: &[ImplInstance::BoundedSig]) -> ImplInstance::BoundedSig {
        ImplInstance::num_linear_combination(self, coeffs, values)
    }

    /// Returns the product. Panics in case of bound overflow.
    fn num_mul(&mut self, a: ImplInstance::BoundedSig, b: ImplInstance::BoundedSig) -> ImplInstance::BoundedSig {
        ImplInstance::num_mul(self, a, b)
    }

    /// Returns the maximal bound that is achievable in a single check.
    /// Check of sizes lesser than this MIGHT be implemented as two checks on x and x-k.
    fn max_primitive_rangecheck(&self) -> usize {
        ImplInstance::max_primitive_rangecheck(self)
    }

    /// Range-checks the signal. Fails if bound > max_primitive_rangecheck
    fn primitive_rangecheck(&mut self, sig: Sig<Self>, bound: &BigUint) -> ImplInstance::BoundedSig {
        ImplInstance::primitive_rangecheck(self, sig, bound)
    }
}


pub struct BigUintRepr<C: Circuit + StandardVariables, R: RangecheckImpl<C>> {
    limbs: Vec<R::BoundedSig>,
    base: BigUint,
}