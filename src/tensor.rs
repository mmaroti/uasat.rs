/*
* Copyright (C) 2019-2020, Miklos Maroti
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

//! Basic multidimensional array type and operations over boolean algebras.

use super::boolean::{BoolAlg, BoolSat};
use super::genvec::{Element, Vector, VectorFor};
use std::ops::Index;

pub use super::boolean::{Boolean, Solver, Trivial};

/// The shape of a tensor.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shape {
    dims: Vec<usize>,
}

impl Shape {
    /// Creates a new shape object.
    pub fn new(dims: Vec<usize>) -> Self {
        Shape { dims }
    }

    /// The number of dimensions.
    pub fn len(self: &Self) -> usize {
        self.dims.len()
    }

    /// Checks if the number of dimensions is zero.
    pub fn is_empty(self: &Self) -> bool {
        self.dims.is_empty()
    }

    /// Checks if all dimensions are equal to the given one.
    pub fn is_rectangular(self: &Self, dim: usize) -> bool {
        self.dims.iter().all(|d| *d == dim)
    }

    /// Returns the head and tail of this shape. The shape must have at
    /// least one dimension.
    pub fn split(self: &Self) -> (usize, Self) {
        assert!(!self.is_empty());
        (self.dims[0], Shape::new(self.dims[1..].to_vec()))
    }

    /// Returns the number of elements this shape represents.
    pub fn size(self: &Self) -> usize {
        let mut size = 1;
        for dim in self.dims.iter() {
            size *= *dim;
        }
        size
    }

    /// Returns a new shape that is the same as this shape but a few new
    /// dimension are inserted into the shape at the given position.
    pub fn insert(self: &Self, pos: usize, dims: &[usize]) -> Self {
        assert!(pos <= self.len());
        let mut dims2 = self.dims.clone();
        dims2.reserve(dims.len());
        for (idx, dim) in dims.iter().enumerate() {
            dims2.insert(pos + idx, *dim);
        }
        Shape { dims: dims2 }
    }

    /// Creates a mapping suitable for a polymer operation where the initial
    /// segment is provided and the rest is filled in starting at the rest
    /// position.
    pub fn mapping(self: &Self, part: &[usize], rest: usize) -> Vec<usize> {
        assert!(part.len() <= self.len());
        let mut mapping = Vec::with_capacity(self.len());
        for dim in part {
            assert!(*dim < rest);
            mapping.push(*dim);
        }
        for i in 0..(self.len() - part.len()) {
            mapping.push(rest + i);
        }
        mapping
    }

    /// Returns the linear index of an element given by coordinates.
    fn index(self: &Self, coords: &[usize]) -> usize {
        assert_eq!(coords.len(), self.len());
        let mut index = 0;
        let mut size = 1;
        for (coord, dim) in coords.iter().zip(self.dims.iter()) {
            assert!(coord < dim);
            index += *coord * size;
            size *= *dim;
        }
        index
    }

    /// Returns the vector of strides for linear indexing
    fn strides(self: &Self) -> Vec<usize> {
        let mut size = 1;
        self.dims
            .iter()
            .map(|d| {
                let s = size;
                size *= d;
                s
            })
            .collect()
    }
}

impl Index<usize> for Shape {
    type Output = usize;

    fn index(self: &Self, idx: usize) -> &Self::Output {
        &self.dims[idx]
    }
}

#[doc(hidden)]
/// Iterator for implementing the polymer operation
struct StrideIter {
    entries: Vec<(usize, usize, usize)>,
    index: usize,
    done: bool,
}

impl StrideIter {
    fn new(shape: &Shape) -> Self {
        let mut done = false;
        let entries = shape
            .dims
            .iter()
            .map(|d| {
                done |= *d == 0;
                (0, *d, 0)
            })
            .collect();

        StrideIter {
            entries,
            index: 0,
            done,
        }
    }

    fn add_stride(self: &mut Self, idx: usize, stride: usize) {
        self.entries[idx].2 += stride;
    }
}

impl Iterator for StrideIter {
    type Item = usize;

    fn next(self: &mut Self) -> Option<usize> {
        if !self.done {
            let index = self.index;
            for entry in self.entries.iter_mut() {
                self.index += entry.2;
                entry.0 += 1;
                if entry.0 >= entry.1 {
                    self.index -= entry.0 * entry.2;
                    entry.0 = 0;
                } else {
                    return Some(index);
                }
            }
            self.done = true;
            return Some(index);
        }
        None
    }
}

/// A multidimensional array of elements.
#[derive(Clone, Debug)]
pub struct Tensor<Elem: Element> {
    shape: Shape,
    elems: VectorFor<Elem>,
}

impl<Elem: Element> Tensor<Elem> {
    /// Creates a tensor of the given shape and with the given elements.
    pub fn new(shape: Shape, elems: VectorFor<Elem>) -> Self {
        assert_eq!(shape.size(), elems.len());
        Tensor { shape, elems }
    }

    /// Creates a tensor filled with constant value
    pub fn constant(shape: Shape, elem: Elem) -> Self {
        let size = shape.size();
        let mut elems: VectorFor<Elem> = Vector::with_capacity(size);
        elems.resize(size, elem);
        Tensor { shape, elems }
    }

    /// Returns the shape of the tensor
    pub fn shape(self: &Self) -> &Shape {
        &self.shape
    }

    /// Returns the element at the given index.
    pub fn very_slow_get(self: &Self, coords: &[usize]) -> Elem {
        self.elems.get(self.shape.index(coords))
    }

    /// Sets the element at the given index.
    pub fn very_slow_set(self: &mut Self, coords: &[usize], elem: Elem) {
        self.elems.set(self.shape.index(coords), elem);
    }

    /// Creates a new tensor of the given shape from the given old tensor with
    /// permuted, identified or new dummy coordinates. The mapping is a vector
    /// of length of the original tensor shape with entries identifying the
    /// matching coordinates in the new tensor.
    pub fn polymer(self: &Self, shape: Shape, mapping: &[usize]) -> Self {
        assert_eq!(mapping.len(), self.shape.len());

        let mut iter = StrideIter::new(&shape);
        let strides = self.shape.strides();
        for (idx, val) in mapping.iter().enumerate() {
            assert_eq!(self.shape[idx], shape[*val]);
            iter.add_stride(*val, strides[idx]);
        }

        let elems: VectorFor<Elem> = iter.map(|i| self.elems.get(i)).collect();
        Tensor::new(shape, elems)
    }

    /// Joins the first `count` many dimensions into a new dimension. The
    /// underlying data will not change, only its shape.
    pub fn reshape_join(self: &mut Self, count: usize) {
        assert!(count <= self.shape.len());

        let mut dims = Vec::with_capacity(1 + self.shape.len() - count);
        dims.push(self.shape.dims[..count].iter().product());
        dims.extend(self.shape.dims[count..].iter());

        self.shape = Shape::new(dims);
    }
}

/// A tensor algebra for tensors.
pub trait TensorAlg {
    /// The type representing the tensor.
    type Elem: Clone;

    /// Returns the shape of the tensor.
    fn shape(elem: &Self::Elem) -> &Shape;

    /// Creates a new scalar tensor for the given element.
    fn scalar(self: &mut Self, elem: bool) -> Self::Elem;

    /// Returns a diagonal tensor of rank two with true elements on the
    /// diagonal and false everywhere else.
    fn diagonal(self: &mut Self, dim: usize) -> Self::Elem;

    /// Creates a new tensor of the given shape from the given old tensor with
    /// permuted, identified or new dummy coordinates. The mapping is a vector
    /// of length of the old tensor shape with entries identifying the
    /// coordinate in the new tensor.
    fn polymer(self: &Self, elem: &Self::Elem, shape: Shape, mapping: &[usize]) -> Self::Elem;

    /// Joins the first `count` many dimensions into a new dimension. The
    /// underlying data will not change, only its shape.
    fn reshape_join(self: &Self, elem: &Self::Elem, count: usize) -> Self::Elem;

    /// Returns a new tensor whose elements are all negated of the original.
    fn tensor_not(self: &mut Self, elem: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor whose elements are disjunctions of the original
    /// elements.
    fn tensor_or(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor whose elements are the conjunctions of the
    /// original elements.
    fn tensor_and(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor whose elements are the boolean additions of the
    /// original elements.
    fn tensor_xor(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor whose elements are the logical equivalence of the
    /// original elements.
    fn tensor_equ(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor whose elements are the logical implication of the
    /// original elements.
    fn tensor_imp(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor with the first dimension removed where the result
    /// is the conjunction of the elements.
    fn tensor_all(self: &mut Self, elem: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor with the first dimension removed where the result
    /// is the disjunction of the elements.
    fn tensor_any(self: &mut Self, elem: &Self::Elem) -> Self::Elem;

    /// Returns a new tensor with the first dimension removed where the result
    /// is the binary sum of the elements.
    fn tensor_sum(self: &mut Self, elem: &Self::Elem) -> Self::Elem;
}

impl<ALG> TensorAlg for ALG
where
    ALG: BoolAlg,
{
    type Elem = Tensor<ALG::Elem>;

    fn shape(elem: &Self::Elem) -> &Shape {
        &elem.shape
    }

    fn scalar(self: &mut Self, elem: bool) -> Self::Elem {
        Tensor::constant(Shape::new(vec![]), self.bool_lift(elem))
    }

    fn diagonal(self: &mut Self, dim: usize) -> Self::Elem {
        let zero = self.bool_zero();
        let mut tensor = Tensor::constant(Shape::new(vec![dim, dim]), zero);

        let unit = self.bool_unit();
        for idx in 0..dim {
            tensor.elems.set(idx * (dim + 1), unit);
        }

        tensor
    }

    fn polymer(self: &Self, tensor: &Self::Elem, shape: Shape, mapping: &[usize]) -> Self::Elem {
        tensor.polymer(shape, mapping)
    }

    fn reshape_join(self: &Self, tensor: &Self::Elem, count: usize) -> Self::Elem {
        let mut tensor = tensor.clone();
        tensor.reshape_join(count);
        tensor
    }

    fn tensor_not(self: &mut Self, tensor: &Self::Elem) -> Self::Elem {
        let elems = tensor.elems.iter().map(|b| self.bool_not(b)).collect();
        Tensor::new(tensor.shape.clone(), elems)
    }

    fn tensor_or(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem {
        assert_eq!(elem1.shape, elem2.shape);
        let elems = elem1
            .elems
            .iter()
            .zip(elem2.elems.iter())
            .map(|(a, b)| self.bool_or(a, b))
            .collect();
        Tensor::new(elem1.shape.clone(), elems)
    }

    fn tensor_and(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem {
        assert_eq!(elem1.shape, elem2.shape);
        let elems = elem1
            .elems
            .iter()
            .zip(elem2.elems.iter())
            .map(|(a, b)| self.bool_and(a, b))
            .collect();
        Tensor::new(elem1.shape.clone(), elems)
    }

    fn tensor_xor(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem {
        assert_eq!(elem1.shape, elem2.shape);
        let elems = elem1
            .elems
            .iter()
            .zip(elem2.elems.iter())
            .map(|(a, b)| self.bool_xor(a, b))
            .collect();
        Tensor::new(elem1.shape.clone(), elems)
    }

    fn tensor_equ(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem {
        assert_eq!(elem1.shape, elem2.shape);
        let elems = elem1
            .elems
            .iter()
            .zip(elem2.elems.iter())
            .map(|(a, b)| self.bool_equ(a, b))
            .collect();
        Tensor::new(elem1.shape.clone(), elems)
    }

    fn tensor_imp(self: &mut Self, elem1: &Self::Elem, elem2: &Self::Elem) -> Self::Elem {
        assert_eq!(elem1.shape, elem2.shape);
        let elems = elem1
            .elems
            .iter()
            .zip(elem2.elems.iter())
            .map(|(a, b)| self.bool_imp(a, b))
            .collect();
        Tensor::new(elem1.shape.clone(), elems)
    }

    fn tensor_all(self: &mut Self, elem: &Self::Elem) -> Self::Elem {
        let (head, shape) = elem.shape.split();
        let elems = (0..shape.size())
            .map(|i| self.bool_all(elem.elems.range(i * head, i * head + head)))
            .collect();

        Tensor::new(shape, elems)
    }

    fn tensor_any(self: &mut Self, elem: &Self::Elem) -> Self::Elem {
        let (head, shape) = elem.shape.split();
        let elems = (0..shape.size())
            .map(|i| self.bool_any(elem.elems.range(i * head, i * head + head)))
            .collect();

        Tensor::new(shape, elems)
    }

    fn tensor_sum(self: &mut Self, elem: &Self::Elem) -> Self::Elem {
        let (head, shape) = elem.shape.split();
        let elems = (0..shape.size())
            .map(|i| self.bool_sum(elem.elems.range(i * head, i * head + head)))
            .collect();

        Tensor::new(shape, elems)
    }
}

/// The trait for solving tensor algebra problems.
pub trait TensorSat: TensorAlg {
    /// Creates a new tensor with fresh variables.
    fn tensor_add_variable(self: &mut Self, shape: Shape) -> Self::Elem;

    /// Adds the given (disjunctive) clause to the solver.
    fn tensor_add_clause(self: &mut Self, elems: &[&Self::Elem]);

    /// Runs the solver and finds a model where the given assumptions are true.
    fn tensor_find_model(self: &mut Self) -> bool;

    /// Returns the logical value of the tensor in the found model.
    fn tensor_get_value(self: &Self, elem: &Self::Elem) -> Tensor<bool>;
}

impl<ALG> TensorSat for ALG
where
    ALG: BoolSat,
{
    fn tensor_add_variable(self: &mut Self, shape: Shape) -> Self::Elem {
        let elems = (0..shape.size())
            .map(|_| self.bool_add_variable())
            .collect();
        Tensor::new(shape, elems)
    }

    fn tensor_add_clause(self: &mut Self, tensors: &[&Self::Elem]) {
        if tensors.is_empty() {
            self.bool_add_clause(&[]);
            return;
        }

        let shape = tensors[0].shape();
        for t in tensors.iter().skip(1) {
            assert_eq!(t.shape(), shape);
        }

        if shape.size() == 0 {
            return;
        }

        let mut clause: Vec<ALG::Elem> = tensors.iter().map(|t| t.elems.get(0)).collect();
        self.bool_add_clause(&clause);

        for i in 1..shape.size() {
            for j in 0..tensors.len() {
                clause[j] = tensors[j].elems.get(i);
            }
            self.bool_add_clause(&clause);
        }
    }

    fn tensor_find_model(self: &mut Self) -> bool {
        self.bool_find_model(&[])
    }

    fn tensor_get_value(self: &Self, tensor: &Self::Elem) -> Tensor<bool> {
        let elems: VectorFor<bool> = tensor
            .elems
            .iter()
            .map(|e| self.bool_get_value(e))
            .collect();
        Tensor::new(tensor.shape.clone(), elems)
    }
}

#[cfg(test)]
mod tests {
    use super::super::boolean::Boolean;
    use super::*;

    #[test]
    fn polymer() {
        let mut tensor: Tensor<usize> = Tensor::constant(Shape::new(vec![2, 3]), 0);
        for i in 0..2 {
            for j in 0..3 {
                tensor.very_slow_set(&[i, j], i + 10 * j);
            }
        }
        let tensor = tensor.polymer(Shape::new(vec![3, 4, 2]), &[2, 0]);
        assert_eq!(tensor.shape, Shape::new(vec![3, 4, 2]));
        for i in 0..2 {
            for j in 0..3 {
                for k in 0..4 {
                    assert_eq!(tensor.very_slow_get(&[j, k, i]), i + 10 * j);
                }
            }
        }
    }

    #[test]
    fn getset() {
        let mut alg = Boolean();
        let mut t1: Tensor<bool> = Tensor::constant(Shape::new(vec![2, 3]), false);
        t1.very_slow_set(&[0, 0], true);
        t1.very_slow_set(&[1, 1], true);
        t1.very_slow_set(&[1, 2], true);

        let t2 = alg.tensor_not(&t1);
        assert_eq!(t2.very_slow_get(&[0, 0]), false);
        assert_eq!(t2.very_slow_get(&[0, 1]), true);

        t1.very_slow_set(&[0, 1], true);
        let t3 = alg.tensor_and(&t1, &t2);
        assert_eq!(t3.very_slow_get(&[0, 0]), false);
        assert_eq!(t3.very_slow_get(&[0, 1]), true);
        assert_eq!(t3.very_slow_get(&[0, 2]), false);
        assert_eq!(t3.very_slow_get(&[1, 0]), false);
        assert_eq!(t3.very_slow_get(&[1, 1]), false);
        assert_eq!(t3.very_slow_get(&[1, 2]), false);
    }

    #[test]
    fn fold() {
        let mut alg = Boolean();
        let mut t1: Tensor<bool> = Tensor::constant(Shape::new(vec![2, 4]), false);
        t1.very_slow_set(&[0, 1], true);
        t1.very_slow_set(&[1, 2], true);
        t1.very_slow_set(&[0, 3], true);
        t1.very_slow_set(&[1, 3], true);

        let t2 = alg.tensor_all(&t1);
        assert_eq!(t2.shape, Shape::new(vec![4]));
        assert_eq!(t2.very_slow_get(&[0]), false);
        assert_eq!(t2.very_slow_get(&[1]), false);
        assert_eq!(t2.very_slow_get(&[2]), false);
        assert_eq!(t2.very_slow_get(&[3]), true);

        let t3 = alg.reshape_join(&t1, 2);
        let t3 = alg.tensor_all(&t3);
        assert_eq!(t3.shape, Shape::new(vec![]));
        assert_eq!(t3.very_slow_get(&[]), false);
    }
}
