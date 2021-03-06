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

//! Module for working with abstract algebras.

#[doc(hidden)]
mod traits;
pub use traits::*;

#[doc(hidden)]
mod two_element_alg;
pub use two_element_alg::*;

#[doc(hidden)]
mod trivial_algebra;
pub use trivial_algebra::*;

#[doc(hidden)]
mod product_algebra;
pub use product_algebra::*;

#[doc(hidden)]
mod free_boolean_alg;
pub use free_boolean_alg::*;

#[doc(hidden)]
mod small_integers;
pub use small_integers::*;

#[doc(hidden)]
mod binary_numbers;
pub use binary_numbers::*;

#[doc(hidden)]
mod binary_vectors;
pub use binary_vectors::*;
