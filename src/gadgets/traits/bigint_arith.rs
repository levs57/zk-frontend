use num_bigint::BigUint;

use crate::circuit::{Circuit, Sig, StandardVariables};

pub trait RangecheckImpl<C: Circuit + StandardVariables> {
    /// Returns upper bound of this signal.
    /// Returns None if no bound is set.
    fn bound(c: &C, sig: Sig<C>) -> Option<BigUint>;

    /// Sets up a new signal bound. If bound is larger than already existing, nothing happens.
    /// MUST fail if bound > field modulus.
    fn assume(c: &mut C, sig: Sig<C>, bound: &BigUint);

    /// Returns non-negative linear combination. Panics in case of bound overflow.
    fn num_linear_combination(c: &mut C, coeffs: &[BigUint], values: &[Sig<C>]) -> Sig<C>;

    /// Returns the product. Panics in case of bound overflow.
    fn num_mul(c: &mut C, a: Sig<C>, b: Sig<C>) -> Sig<C>;

    /// Returns the maximal bound that is achievable in a single check.
    /// Check of sizes lesser than this MIGHT be implemented as two checks on x and x-k.
    fn max_primitive_rangecheck(c: &C) -> usize;

    /// Range-checks the signal. Fails if bound > max_primitive_rangecheck
    fn primitive_rangecheck(c: &mut C, sig: Sig<C>, bound: &BigUint);

    /// Splits the value into limbs with some base. Does not constrain anything.
    fn advise_split_into_n_limbs(
        c: &mut C,
        sig: Sig<C>,
        base: &BigUint,
        num_limbs: u32,
    ) -> Vec<Sig<C>>;

    /// Splits the signal into limbs (and range-checks it). Each limb is combined from #packing primitive limbs.
    /// Should fail if primitive_base > max_primitive_rangecheck.
    /// For base = primitive_base^base_pow requires base^num_limbs < field modulus.
    fn split_into_n_limbs(
        c: &mut C,
        sig: Sig<C>,
        primitive_base: &BigUint,
        packing: u32,
        num_limbs: u32,
    ) -> Vec<Sig<C>> {
        let base = primitive_base.pow(packing);
        let bound = &base.pow(num_limbs);
        let primitive_limbs =
            Self::advise_split_into_n_limbs(c, sig, primitive_base, num_limbs * packing);
        primitive_limbs
            .iter()
            .map(|sig| Self::primitive_rangecheck(c, *sig, primitive_base))
            .count();
        Self::assume(c, sig, &bound);

        let mut coeffs = vec![];
        let mut power = BigUint::from(1u32);
        for _ in 0..packing {
            coeffs.push(power.clone());
            power *= primitive_base;
        }

        let mut limbs = vec![];
        for i in 0..num_limbs {
            let mut lc = vec![];
            for j in 0..packing {
                lc.push(primitive_limbs[(i * packing + j) as usize]);
            }
            let limb = Self::num_linear_combination(c, &coeffs, &lc);
            limbs.push(limb);
        }

        limbs
    }

    /// Splits already bounded signal into appropriate amount of limbs.
    /// Might fail if bound is too close to field modulus.
    fn split_into_limbs_strict(
        c: &mut C,
        sig: Sig<C>,
        primitive_base: &BigUint,
        packing: u32,
    ) -> Vec<Sig<C>> {
        let base = primitive_base.pow(packing);
        let num_limbs = log_ceil(&base, &Self::bound(c, sig).unwrap());
        Self::split_into_n_limbs(c, sig, primitive_base, packing, num_limbs)
    }

    /// Attempts to flush the registers. Might fail if bounds are too large.
    /// WARNING: this implementation does not optimize linear constraints; optimally this should be done by backend.
    fn normalize(
        c: &mut C,
        limbs: &Vec<Sig<C>>,
        primitive_base: &BigUint,
        packing: u32,
    ) -> Vec<Sig<C>> {
        let mut limbs: Vec<_> = limbs.into_iter().map(|x| vec![*x]).collect();
        let mut i = 0;
        loop {
            let incoming = Self::num_linear_combination(
                c,
                &vec![BigUint::from(1u32); limbs[i].len()],
                &limbs[i],
            );
            limbs[i] = vec![];
            let term = Self::split_into_limbs_strict(c, incoming, primitive_base, packing);
            for j in 0..term.len() {
                if limbs.len() > i + j {
                    limbs[i + j].push(term[j]);
                } else if limbs.len() == i + j {
                    limbs.push(vec![term[j]]);
                } else {
                    panic!();
                }
            }
            i += 1;
            if limbs.len() <= i {
                break;
            }
        }
        limbs
            .iter()
            .map(|x| if x.len() > 1 { panic!() } else { x[0] })
            .collect()
    }
}

/// Returns ceil(log_b(x)).
/// Can be used to compute amount of limbs of base b necessary to hold any value in 0..x.
fn log_ceil(b: &BigUint, x: &BigUint) -> u32 {
    let mut pows = vec![b.clone()]; // powers b^{2^k}
    loop {
        let l = pows.len();
        let pow_new = (&pows[l - 1]) * (&pows[l - 1]);
        if &pow_new > x {
            break;
        } else {
            pows.push(pow_new);
        }
    }
    let l = pows.len();
    let mut ret = 0;
    let mut approx = BigUint::from(1u32);
    for i in 0..l {
        let k = l - i - 1;
        let approx_new = &approx * &pows[k];
        if &approx_new < x {
            approx = approx_new;
            ret += 1 << k;
        }
    }

    ret + 1
}

pub trait Rangecheck: Circuit + StandardVariables {
    type IRangecheck: RangecheckImpl<Self>;

    /// Returns upper bound of this signal.
    /// Returns None if no bound is set.
    fn bound(&self, sig: Sig<Self>) -> Option<BigUint> {
        Self::IRangecheck::bound(self, sig)
    }

    /// Sets up a new signal bound. If bound is larger than already existing, nothing happens.
    /// MUST fail if bound > field modulus.
    fn assume(&mut self, sig: Sig<Self>, bound: &BigUint) {
        Self::IRangecheck::assume(self, sig, bound)
    }

    /// Returns non-negative linear combination. Panics in case of bound overflow.
    fn num_linear_combination(&mut self, coeffs: &[BigUint], values: &[Sig<Self>]) -> Sig<Self> {
        Self::IRangecheck::num_linear_combination(self, coeffs, values)
    }

    /// Returns the product. Panics in case of bound overflow.
    fn num_mul(&mut self, a: Sig<Self>, b: Sig<Self>) -> Sig<Self> {
        Self::IRangecheck::num_mul(self, a, b)
    }

    /// Returns the maximal bound that is achievable in a single check.
    /// Check of sizes lesser than this MIGHT be implemented as two checks on x and x-k.
    fn max_primitive_rangecheck(&self) -> usize {
        Self::IRangecheck::max_primitive_rangecheck(self)
    }

    /// Range-checks the signal. Fails if bound > max_primitive_rangecheck
    fn primitive_rangecheck(&mut self, sig: Sig<Self>, bound: &BigUint) {
        Self::IRangecheck::primitive_rangecheck(self, sig, bound)
    }

    /// Splits the value into limbs with some base. Does not constrain anything.
    fn advise_split_into_n_limbs(
        &mut self,
        sig: Sig<Self>,
        base: &BigUint,
        num_limbs: u32,
    ) -> Vec<Sig<Self>> {
        Self::IRangecheck::advise_split_into_n_limbs(self, sig, base, num_limbs)
    }

    /// Splits the signal into limbs (and range-checks it). Each limb is combined from #packing primitive limbs.
    /// Should fail if primitive_base > max_primitive_rangecheck.
    /// For base = primitive_base^base_pow requires base^num_limbs < field modulus.
    fn split_into_n_limbs(
        &mut self,
        sig: Sig<Self>,
        primitive_base: &BigUint,
        packing: u32,
        num_limbs: u32,
    ) -> Vec<Sig<Self>> {
        Self::IRangecheck::split_into_n_limbs(self, sig, primitive_base, packing, num_limbs)
    }

    /// Splits already bounded signal into appropriate amount of limbs.
    /// Might fail if bound is too close to field modulus.
    fn split_into_limbs_strict(
        &mut self,
        sig: Sig<Self>,
        primitive_base: &BigUint,
        packing: u32,
    ) -> Vec<Sig<Self>> {
        Self::IRangecheck::split_into_limbs_strict(self, sig, primitive_base, packing)
    }
}

//     /// Returns upper bound of this signal.
//     fn bound(&self, sig: ImplInstance::BoundedSig) -> BigUint {
//         ImplInstance::bound(self, sig)
//     }

//     /// Increases bound to a new one. MUST fail if the suggested bound is smaller.
//     /// Note that while sig is immutable, it is an address, which points to the actual bound data.
//     fn promote(&mut self, sig: ImplInstance::BoundedSig, new_bound: &BigUint) {
//         ImplInstance::promote(self, sig, new_bound)
//     }

//     /// Converts the signal into the bounded one, assuming the bound.
//     /// Ideally, should not require a copy constraint.
//     fn assume_bounded(&mut self, sig: Sig<Self>, bound: &BigUint) -> Option<ImplInstance::BoundedSig> {
//         ImplInstance::assume_bounded(self, sig, bound)
//     }

//     /// Returns normal signal representation.
//     /// Ideally, should not require a copy constraint.
//     fn as_sig(&self, sig: ImplInstance::BoundedSig) -> Sig<Self> {
//         ImplInstance::as_sig(self, sig)
//     }

//     /// Returns non-negative linear combination. Panics in case of bound overflow.
//     fn num_linear_combination(&mut self, coeffs: &[BigUint], values: &[ImplInstance::BoundedSig]) -> ImplInstance::BoundedSig {
//         ImplInstance::num_linear_combination(self, coeffs, values)
//     }

//     /// Returns the product. Panics in case of bound overflow.
//     fn num_mul(&mut self, a: ImplInstance::BoundedSig, b: ImplInstance::BoundedSig) -> ImplInstance::BoundedSig {
//         ImplInstance::num_mul(self, a, b)
//     }

//     /// Returns the maximal bound that is achievable in a single check.
//     /// Check of sizes lesser than this MIGHT be implemented as two checks on x and x-k.
//     fn max_primitive_rangecheck(&self) -> usize {
//         ImplInstance::max_primitive_rangecheck(self)
//     }

//     /// Range-checks the signal. Fails if bound > max_primitive_rangecheck
//     fn primitive_rangecheck(&mut self, sig: Sig<Self>, bound: &BigUint) -> ImplInstance::BoundedSig {
//         ImplInstance::primitive_rangecheck(self, sig, bound)
//     }
// }
