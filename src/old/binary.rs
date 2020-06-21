/*
* Copyright (C) 2020, Miklos Maroti
*
* This program is free software: you can redistribute it and/or modify
* it under the terms of the GNU General Public License as published by
* the Free Software Foundation, either version 3 of the License, or
* (at your option) any later version.
*
* This program is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
* GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License
* along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use crate::boolean;
use crate::genvec;
use crate::genvec::Vector as _;

pub use boolean::{Boolean, Solver, Trivial};

/// Boolean array algebra representing bit vectors and binary numbers.
pub trait BinaryAlg {
    type Elem;

    /// Returns the length of the array.
    fn len(&self, elem: &Self::Elem) -> usize;

    /// Concatenates the given vectors into a single one.
    fn concat(&self, elems: Vec<Self::Elem>) -> Self::Elem;

    /// Splits this vector into equal sized vectors
    fn split(&self, elem: Self::Elem, len: usize) -> Vec<Self::Elem>;

    /// Creates a new vector of the given length containing the element.
    fn bit_lift(&mut self, elem: &[bool]) -> Self::Elem;

    /// Returns the element wise negation of the vector.
    fn bit_not(&mut self, elem: Self::Elem) -> Self::Elem;

    /// Returns a new vector whose elements are disjunctions of the original
    /// elements.
    fn bit_or(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem;

    /// Returns a new vector whose elements are conjunction of the original
    /// elements.
    fn bit_and(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let elem2 = self.bit_not(elem2);
        let elem3 = self.bit_imp(elem1, elem2);
        self.bit_not(elem3)
    }

    /// Returns a new vector whose elements are the exclusive or of the
    /// original elements.
    fn bit_xor(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem;

    /// Returns a new vector whose elements are the are the logical equivalence
    /// of the original elements.
    fn bit_equ(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let elem1 = self.bit_not(elem1);
        self.bit_xor(elem1, elem2)
    }

    /// Returns a new vector whose elements are the are the logical implication
    /// of the original elements.
    fn bit_imp(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let elem1 = self.bit_not(elem1);
        self.bit_or(elem1, elem2)
    }

    /// Returns the conjunction of all elements as a 1-element vector.
    fn bit_all(&mut self, elem: Self::Elem) -> Self::Elem;

    /// Returns the conjunction of all elements as a 1-element vector.
    fn bit_any(&mut self, elem: Self::Elem) -> Self::Elem {
        let elem = self.bit_not(elem);
        let elem = self.bit_all(elem);
        self.bit_not(elem)
    }

    /// Creates a new vector of the given length representing the given binary
    /// number.
    fn num_lift(&self, len: usize, elem: i64) -> Self::Elem;

    /// Returns the negative of the given binary number in two's complement.
    fn num_neg(&mut self, elem: Self::Elem) -> Self::Elem {
        let elem = self.bit_not(elem);
        let one = self.num_lift(self.len(&elem), 1);
        self.num_add(elem, one)
    }

    /// Returns the sum of the two binary numbers of the same length in
    /// two's complement.
    fn num_add(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem;

    /// Returns the difference of the two binary numbers of the same length in
    /// two's complement.
    fn num_sub(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let elem2 = self.num_neg(elem2);
        self.num_add(elem1, elem2)
    }

    /// Returns whether the first binary number is equal to the second one
    /// as a 1-element vector.
    fn num_eq(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let elem3 = self.bit_equ(elem1, elem2);
        self.bit_all(elem3)
    }

    /// Returns whether the first binary number is not equal to the second one
    /// as a 1-element vector.
    fn num_ne(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let tmp = self.num_eq(elem1, elem2);
        self.bit_not(tmp)
    }

    /// Returns whether the first unsigned binary number is less than or equal
    /// to the second one as a 1-element vector.
    fn num_le(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem;

    /// Returns whether the first unsigned binary number is less than the
    /// second one as a 1-element vector.
    fn num_lt(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let tmp = self.num_le(elem2, elem1);
        self.bit_not(tmp)
    }
}

impl<ALG> BinaryAlg for ALG
where
    ALG: boolean::BoolAlg,
    ALG::Elem: genvec::Element,
{
    type Elem = genvec::VectorFor<ALG::Elem>;

    fn len(&self, elem: &Self::Elem) -> usize {
        elem.len()
    }

    fn concat(&self, elems: Vec<Self::Elem>) -> Self::Elem {
        genvec::Vector::concat(elems)
    }

    fn split(&self, elem: Self::Elem, len: usize) -> Vec<Self::Elem> {
        genvec::Vector::split(elem, len)
    }

    fn bit_lift(&mut self, elem: &[bool]) -> Self::Elem {
        elem.iter().map(|a| self.bool_lift(*a)).collect()
    }

    fn bit_not(&mut self, elem: Self::Elem) -> Self::Elem {
        elem.iter().map(|a| self.bool_not(a)).collect()
    }

    fn bit_or(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| self.bool_or(a, b))
            .collect()
    }

    fn bit_and(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| self.bool_and(a, b))
            .collect()
    }

    fn bit_xor(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| self.bool_xor(a, b))
            .collect()
    }

    fn bit_equ(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| self.bool_equ(a, b))
            .collect()
    }

    fn bit_imp(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| self.bool_imp(a, b))
            .collect()
    }

    fn bit_all(&mut self, elem: Self::Elem) -> Self::Elem {
        let mut result = self.bool_unit();
        for a in elem.iter() {
            result = self.bool_and(result, a);
        }
        genvec::Vector::from_elem(result)
    }

    fn bit_any(&mut self, elem: Self::Elem) -> Self::Elem {
        let mut result = self.bool_zero();
        for a in elem.iter() {
            result = self.bool_or(result, a);
        }
        genvec::Vector::from_elem(result)
    }

    fn num_lift(&self, len: usize, elem: i64) -> Self::Elem {
        (0..len)
            .map(|i| self.bool_lift((elem >> i) & 1 != 0))
            .collect()
    }

    fn num_neg(&mut self, elem: Self::Elem) -> Self::Elem {
        let mut carry = self.bool_unit();
        elem.iter()
            .map(|a| {
                let b = self.bool_not(a);
                let c = self.bool_xor(b, carry);
                carry = self.bool_and(b, carry);
                c
            })
            .collect()
    }

    fn num_add(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        let mut carry = self.bool_zero();
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| {
                let c = self.bool_sum3(a, b, carry);
                carry = self.bool_maj(a, b, carry);
                c
            })
            .collect()
    }

    fn num_sub(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        let mut carry = self.bool_unit();
        elem1
            .iter()
            .zip(elem2.iter())
            .map(|(a, b)| {
                let b = self.bool_not(b);
                let c = self.bool_sum3(a, b, carry);
                carry = self.bool_maj(a, b, carry);
                c
            })
            .collect()
    }

    fn num_eq(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        let mut result = self.bool_unit();
        for (a, b) in elem1.iter().zip(elem2.iter()) {
            let c = self.bool_equ(a, b);
            result = self.bool_and(result, c);
        }
        genvec::Vector::from_elem(result)
    }

    fn num_ne(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let mut elem = self.num_eq(elem1, elem2);
        elem.set(0, self.bool_not(elem.get(0)));
        elem
    }

    fn num_le(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        assert_eq!(elem1.len(), elem2.len());
        let mut result = self.bool_unit();
        for (a, b) in elem1.iter().zip(elem2.iter()) {
            let a = self.bool_not(a);
            result = self.bool_maj(a, b, result);
        }
        genvec::Vector::from_elem(result)
    }

    fn num_lt(&mut self, elem1: Self::Elem, elem2: Self::Elem) -> Self::Elem {
        let mut elem = self.num_le(elem2, elem1);
        elem.set(0, self.bool_not(elem.get(0)));
        elem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opers() {
        let alg = Trivial();
        let v1 = alg.num_lift(3, 13);
        assert_eq!(v1, (0..3).map(|_| ()).collect());

        let mut alg = Boolean();
        for a1 in 0..15 {
            let a2 = alg.num_lift(4, a1);
            assert_eq!(a2, alg.num_lift(4, a1 - 16));
            assert_eq!(alg.bit_not(a2.clone()), alg.num_lift(4, !a1));
            assert_eq!(alg.num_neg(a2.clone()), alg.num_lift(4, -a1));
            assert_eq!(alg.concat(vec![a2.clone()]), a2.clone());

            for b1 in 0..15 {
                let b2 = alg.num_lift(4, b1);
                assert_eq!(
                    alg.bit_and(a2.clone(), b2.clone()),
                    alg.num_lift(4, a1 & b1)
                );
                assert_eq!(alg.bit_or(a2.clone(), b2.clone()), alg.num_lift(4, a1 | b1));
                assert_eq!(
                    alg.bit_xor(a2.clone(), b2.clone()),
                    alg.num_lift(4, a1 ^ b1)
                );
                assert_eq!(
                    alg.bit_equ(a2.clone(), b2.clone()),
                    alg.num_lift(4, !a1 ^ b1)
                );
                assert_eq!(
                    alg.bit_imp(a2.clone(), b2.clone()),
                    alg.num_lift(4, !a1 | b1)
                );

                assert_eq!(
                    alg.num_add(a2.clone(), b2.clone()),
                    alg.num_lift(4, a1 + b1)
                );
                assert_eq!(
                    alg.num_sub(a2.clone(), b2.clone()),
                    alg.num_lift(4, a1 - b1)
                );
                assert_eq!(
                    alg.num_eq(a2.clone(), b2.clone()),
                    alg.bit_lift(&[a1 == b1])
                );
                assert_eq!(
                    alg.num_ne(a2.clone(), b2.clone()),
                    alg.bit_lift(&[a1 != b1])
                );
                assert_eq!(
                    alg.num_le(a2.clone(), b2.clone()),
                    alg.bit_lift(&[a1 <= b1])
                );
                assert_eq!(alg.num_lt(a2.clone(), b2.clone()), alg.bit_lift(&[a1 < b1]));

                assert_eq!(
                    alg.concat(vec![a2.clone(), b2.clone()]),
                    alg.num_lift(8, a1 + 16 * b1)
                );
            }
        }
    }
}