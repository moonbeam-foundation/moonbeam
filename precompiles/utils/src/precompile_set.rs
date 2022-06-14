// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Provide utils assemble precompiles and precompilesets into a
//! final precompile set with security checks. All security checks are enabled by
//! default and must be disabled explicely throught type annotations.

use crate::{revert, SelfRefPrecompile};
use fp_evm::{Precompile, PrecompileHandle, PrecompileResult, PrecompileSet};
use frame_support::pallet_prelude::Get;
use impl_trait_for_tuples::impl_for_tuples;
use pallet_evm::AddressMapping;
use sp_core::H160;
use sp_std::{cell::RefCell, marker::PhantomData, vec, vec::Vec};

// CONFIGURATION TYPES

mod sealed {
	pub trait Sealed {}
}

/// How much recursion is allows for a precompile.
pub trait RecursionLimit: sealed::Sealed {
	fn recursion_limit() -> Option<u16>;
}

/// There is no limit to the amount times a precompiles can
/// call itself recursively.
/// Should be used with care as it could cause stack overflows.
pub struct NoRecursionLimit;
impl sealed::Sealed for NoRecursionLimit {}
impl RecursionLimit for NoRecursionLimit {
	#[inline(always)]
	fn recursion_limit() -> Option<u16> {
		None
	}
}

/// A precompile can be called recursively N times.
pub struct LimitRecursionTo<const N: u16>;
impl<const N: u16> sealed::Sealed for LimitRecursionTo<N> {}
impl<const N: u16> RecursionLimit for LimitRecursionTo<N> {
	#[inline(always)]
	fn recursion_limit() -> Option<u16> {
		Some(N)
	}
}

/// Is DELEGATECALL allowed to use for a precompile.
pub trait DelegateCallSupport: sealed::Sealed {
	fn allow_delegate_call() -> bool;
}

/// DELEGATECALL is forbiden.
pub struct ForbidDelegateCall;
impl sealed::Sealed for ForbidDelegateCall {}
impl DelegateCallSupport for ForbidDelegateCall {
	#[inline(always)]
	fn allow_delegate_call() -> bool {
		false
	}
}

/// DELEGATECALL is allowed.
/// Should be used with care if the precompile use
/// custom storage, as the caller could impersonate its own caller.
pub struct AllowDelegateCall;
impl sealed::Sealed for AllowDelegateCall {}
impl DelegateCallSupport for AllowDelegateCall {
	#[inline(always)]
	fn allow_delegate_call() -> bool {
		true
	}
}

pub struct AddressU64<const N: u64>;
impl<const N: u64> Get<H160> for AddressU64<N> {
	#[inline(always)]
	fn get() -> H160 {
		H160::from_low_u64_be(N)
	}
}

// INDIVIDUAL PRECOMPILE(SET)

/// A fragment of a PrecompileSet. Should be implemented as is it
/// was a PrecompileSet containing only the precompile(set) it wraps.
/// They can be combined into a real PrecompileSet using `PrecompileSetFromTuple`.
pub trait PrecompileSetFragment {
	/// Instanciate the fragment.
	fn new() -> Self;

	/// Execute the fragment.
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult>;

	/// Is the provided address a precompile in this fragment?
	fn is_precompile(&self, address: H160) -> bool;

	/// Return the list of addresses covered by this fragment.
	fn used_addresses(&self) -> Vec<H160>;
}

/// Wraps a stateless precompile: a type implementing the `Precompile` trait.
/// Type parameters allow to define:
/// - A: The address of the precompile
/// - R: The recursion limit (defaults to 1)
/// - R: If DELEGATECALL is supported (default to no)
pub struct PrecompileAt<A, P, R = LimitRecursionTo<1>, D = ForbidDelegateCall> {
	current_recursion_level: RefCell<u16>,
	_phantom: PhantomData<(A, P, R, D)>,
}

impl<A, P, R, D> PrecompileSetFragment for PrecompileAt<A, P, R, D>
where
	A: Get<H160>,
	P: Precompile,
	R: RecursionLimit,
	D: DelegateCallSupport,
{
	#[inline(always)]
	fn new() -> Self {
		Self {
			current_recursion_level: RefCell::new(0),
			_phantom: PhantomData,
		}
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		// Check if this is the address of the precompile.
		if A::get() != handle.code_address() {
			return None;
		}

		// Check DELEGATECALL config.
		if !D::allow_delegate_call() && handle.code_address() != handle.context().address {
			return Some(Err(revert(
				"cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		// Check and increase recursion level if needed.
		if let Some(max_recursion_level) = R::recursion_limit() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					if *recursion_level >= max_recursion_level {
						return Some(Err(revert("precompile is called with too high nesting")));
					}

					*recursion_level += 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("couldn't check precompile nesting"))),
			}
		}

		let res = P::execute(handle);

		// Decrease recursion level if needed.
		if R::recursion_limit().is_some() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					*recursion_level -= 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("couldn't check precompile nesting"))),
			}
		}

		Some(res)
	}

	#[inline(always)]
	fn is_precompile(&self, address: H160) -> bool {
		address == A::get()
	}

	#[inline(always)]
	fn used_addresses(&self) -> Vec<H160> {
		vec![A::get()]
	}
}

/// Wraps a stateful precompile: a type implementing the `StatefulPrecompile` trait.
/// Type parameters allow to define:
/// - A: The address of the precompile
/// - R: The recursion limit (defaults to 1)
/// - R: If DELEGATECALL is supported (default to no)
pub struct SelfRefPrecompileAt<A, P, R = LimitRecursionTo<1>, D = ForbidDelegateCall> {
	precompile: P,
	current_recursion_level: RefCell<u16>,
	_phantom: PhantomData<(A, R, D)>,
}

impl<A, P, R, D> PrecompileSetFragment for SelfRefPrecompileAt<A, P, R, D>
where
	A: Get<H160>,
	P: SelfRefPrecompile,
	R: RecursionLimit,
	D: DelegateCallSupport,
{
	#[inline(always)]
	fn new() -> Self {
		Self {
			precompile: P::instanciate(),
			current_recursion_level: RefCell::new(0),
			_phantom: PhantomData,
		}
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		// Check if this is the address of the precompile.
		if A::get() != handle.code_address() {
			return None;
		}

		// Check DELEGATECALL config.
		if !D::allow_delegate_call() && handle.code_address() != handle.context().address {
			return Some(Err(revert(
				"cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		// Check and increase recursion level if needed.
		if let Some(max_recursion_level) = R::recursion_limit() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					if *recursion_level >= max_recursion_level {
						return Some(Err(revert("precompile is called with too high nesting")));
					}

					*recursion_level += 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("couldn't check precompile nesting"))),
			}
		}

		let res = self.precompile.execute(handle);

		// Decrease recursion level if needed.
		if R::recursion_limit().is_some() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					*recursion_level -= 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("couldn't check precompile nesting"))),
			}
		}

		Some(res)
	}

	#[inline(always)]
	fn is_precompile(&self, address: H160) -> bool {
		address == A::get()
	}

	#[inline(always)]
	fn used_addresses(&self) -> Vec<H160> {
		vec![A::get()]
	}
}

/// Wraps an inner PrecompileSet with all its addresses starting with
/// a common prefix.
/// Type parameters allow to define:
/// - A: The common prefix
/// - D: If DELEGATECALL is supported (default to no)
pub struct PrecompileSetStartingWith<A, P, D = ForbidDelegateCall> {
	precompile_set: P,
	_phantom: PhantomData<(A, D)>,
}

impl<A, P, D> PrecompileSetFragment for PrecompileSetStartingWith<A, P, D>
where
	A: Get<&'static [u8]>,
	P: PrecompileSet + Default,
	D: DelegateCallSupport,
{
	#[inline(always)]
	fn new() -> Self {
		Self {
			precompile_set: P::default(),
			_phantom: PhantomData,
		}
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		if self.is_precompile(handle.code_address()) {
			// Check DELEGATECALL config.
			if !D::allow_delegate_call() && handle.code_address() != handle.context().address {
				return Some(Err(revert(
					"cannot be called with DELEGATECALL or CALLCODE",
				)));
			}

			self.precompile_set.execute(handle)
		} else {
			None
		}
	}

	#[inline(always)]
	fn is_precompile(&self, address: H160) -> bool {
		address.as_bytes().starts_with(A::get()) && self.precompile_set.is_precompile(address)
	}

	#[inline(always)]
	fn used_addresses(&self) -> Vec<H160> {
		// TODO: We currently can't get the list of used addresses.
		vec![]
	}
}

/// Make a precompile that always revert.
/// Can be useful when writing tests.
pub struct RevertPrecompile<A>(PhantomData<A>);

impl<A> PrecompileSetFragment for RevertPrecompile<A>
where
	A: Get<H160>,
{
	#[inline(always)]
	fn new() -> Self {
		Self(PhantomData)
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		if A::get() == handle.code_address() {
			Some(Err(revert("revert")))
		} else {
			None
		}
	}

	#[inline(always)]
	fn is_precompile(&self, address: H160) -> bool {
		address == A::get()
	}

	#[inline(always)]
	fn used_addresses(&self) -> Vec<H160> {
		vec![A::get()]
	}
}

// COMPOSITION OF PARTS
#[impl_for_tuples(1, 100)]
impl PrecompileSetFragment for Tuple {
	#[inline(always)]
	fn new() -> Self {
		(for_tuples!(#(
			Tuple::new()
		),*))
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		for_tuples!(#(
			if let Some(res) = self.Tuple.execute(handle) {
				return Some(res);
			}
		)*);

		None
	}

	#[inline(always)]
	fn is_precompile(&self, address: H160) -> bool {
		for_tuples!(#(
			if self.Tuple.is_precompile(address) {
				return true;
			}
		)*);

		false
	}

	#[inline(always)]
	fn used_addresses(&self) -> Vec<H160> {
		let mut used_addresses = vec![];

		for_tuples!(#(
			let mut inner = self.Tuple.used_addresses();
			used_addresses.append(&mut inner);
		)*);

		used_addresses
	}
}

/// Wraps a tuple of `PrecompileSetFragment` to make a real `PrecompileSet`.
pub struct PrecompileSetFromTuple<R, P> {
	inner: P,
	_phantom: PhantomData<R>,
}

impl<R, P: PrecompileSetFragment> PrecompileSet for PrecompileSetFromTuple<R, P> {
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		self.inner.execute(handle)
	}

	fn is_precompile(&self, address: H160) -> bool {
		self.inner.is_precompile(address)
	}
}

impl<R: pallet_evm::Config, P: PrecompileSetFragment> PrecompileSetFromTuple<R, P> {
	/// Cretate a new instance of the PrecompileSet.
	pub fn new() -> Self {
		Self {
			inner: P::new(),
			_phantom: PhantomData,
		}
	}

	/// Return the list of addresses contained in this PrecompileSet.
	pub fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		Self::new()
			.inner
			.used_addresses()
			.into_iter()
			.map(|x| R::AddressMapping::into_account_id(x))
	}
}
