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

use crate::revert;
use fp_evm::{Precompile, PrecompileHandle, PrecompileResult, PrecompileSet};
use frame_support::pallet_prelude::Get;
use impl_trait_for_tuples::impl_for_tuples;
use pallet_evm::AddressMapping;
use sp_core::{H160, H256};
use sp_std::{
	cell::RefCell, collections::btree_map::BTreeMap, marker::PhantomData, ops::RangeInclusive, vec,
	vec::Vec,
};

/// Trait representing checks that can be made on a precompile call.
/// Types implementing this trait are made to be chained in a tuple.
/// For that reason every method returns an Option, None meaning that
/// the implementor have no constraint and the decision is left to
/// latter elements in the chain. If None is returned by all elements of
/// the chain then sensible defaults are used.
pub trait PrecompileChecks {
	#[inline(always)]
	fn recursion_limit() -> Option<Option<u16>> {
		None
	}

	#[inline(always)]
	fn supports_delegate_call() -> Option<bool> {
		None
	}

	#[inline(always)]
	fn callable_by_smart_contract() -> Option<bool> {
		None
	}

	#[inline(always)]
	fn allow_subcalls() -> Option<bool> {
		None
	}
}

#[impl_for_tuples(0, 20)]
impl PrecompileChecks for Tuple {
	#[inline(always)]
	fn recursion_limit() -> Option<Option<u16>> {
		for_tuples!(#(
			if let Some(check) = Tuple::recursion_limit() {
				return Some(check);
			}
		)*);

		None
	}

	#[inline(always)]
	fn supports_delegate_call() -> Option<bool> {
		for_tuples!(#(
			if let Some(check) = Tuple::supports_delegate_call() {
				return Some(check);
			}
		)*);

		None
	}

	#[inline(always)]
	fn callable_by_smart_contract() -> Option<bool> {
		for_tuples!(#(
			if let Some(check) = Tuple::callable_by_smart_contract() {
				return Some(check);
			}
		)*);

		None
	}

	#[inline(always)]
	fn allow_subcalls() -> Option<bool> {
		for_tuples!(#(
			if let Some(check) = Tuple::allow_subcalls() {
				return Some(check);
			}
		)*);

		None
	}
}

pub struct DelegateCallable;

impl PrecompileChecks for DelegateCallable {
	#[inline(always)]
	fn supports_delegate_call() -> Option<bool> {
		Some(true)
	}
}

pub struct SubcallWithMaxNesting<const R: u16>;

impl<const R: u16> PrecompileChecks for SubcallWithMaxNesting<R> {
	#[inline(always)]
	fn recursion_limit() -> Option<Option<u16>> {
		Some(Some(R))
	}

	#[inline(always)]
	fn allow_subcalls() -> Option<bool> {
		Some(true)
	}
}

pub struct AddressU64<const N: u64>;
impl<const N: u64> Get<H160> for AddressU64<N> {
	#[inline(always)]
	fn get() -> H160 {
		H160::from_low_u64_be(N)
	}
}

pub struct RestrictiveHandle<'a, H> {
	handle: &'a mut H,
	allow_subcalls: bool,
}

impl<'a, H: PrecompileHandle> PrecompileHandle for RestrictiveHandle<'a, H> {
	fn call(
		&mut self,
		address: H160,
		transfer: Option<evm::Transfer>,
		input: Vec<u8>,
		target_gas: Option<u64>,
		is_static: bool,
		context: &evm::Context,
	) -> (evm::ExitReason, Vec<u8>) {
		if !self.allow_subcalls {
			return (
				evm::ExitReason::Revert(evm::ExitRevert::Reverted),
				crate::encoded_revert("subcalls disabled for this precompile"),
			);
		}

		self.handle
			.call(address, transfer, input, target_gas, is_static, context)
	}

	fn record_cost(&mut self, cost: u64) -> Result<(), evm::ExitError> {
		self.handle.record_cost(cost)
	}

	fn remaining_gas(&self) -> u64 {
		self.handle.remaining_gas()
	}

	fn log(
		&mut self,
		address: H160,
		topics: Vec<H256>,
		data: Vec<u8>,
	) -> Result<(), evm::ExitError> {
		self.handle.log(address, topics, data)
	}

	fn code_address(&self) -> H160 {
		self.handle.code_address()
	}

	fn input(&self) -> &[u8] {
		self.handle.input()
	}

	fn context(&self) -> &evm::Context {
		self.handle.context()
	}

	fn is_static(&self) -> bool {
		self.handle.is_static()
	}

	fn gas_limit(&self) -> Option<u64> {
		self.handle.gas_limit()
	}
}

// INDIVIDUAL PRECOMPILE(SET)

/// A fragment of a PrecompileSet. Should be implemented as is it
/// was a PrecompileSet containing only the precompile(set) it wraps.
/// They can be combined into a real PrecompileSet using `PrecompileSetBuilder`.
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
/// - D: If DELEGATECALL is supported (default to no)
pub struct PrecompileAt<A, P, C = ()> {
	current_recursion_level: RefCell<u16>,
	_phantom: PhantomData<(A, P, C)>,
}

impl<A, P, C> PrecompileSetFragment for PrecompileAt<A, P, C>
where
	A: Get<H160>,
	P: Precompile,
	C: PrecompileChecks,
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
		let code_address = handle.code_address();

		// Check if this is the address of the precompile.
		if A::get() != code_address {
			return None;
		}

		// Check DELEGATECALL config.
		let allow_delegate_call = C::supports_delegate_call().unwrap_or(false);
		if !allow_delegate_call && code_address != handle.context().address {
			return Some(Err(revert(
				"Cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		// Check and increase recursion level if needed.
		let recursion_limit = C::recursion_limit().unwrap_or(Some(0));
		if let Some(max_recursion_level) = recursion_limit {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					if *recursion_level > max_recursion_level {
						return Some(Err(
							revert("Precompile is called with too high nesting").into()
						));
					}

					*recursion_level += 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("Couldn't check precompile nesting").into())),
			}
		}

		// TODO: Check called by contract + subcall protection
		let allow_subcalls = C::allow_subcalls().unwrap_or(false);
		let mut handle = RestrictiveHandle {
			handle,
			allow_subcalls,
		};

		let res = P::execute(&mut handle);

		// Decrease recursion level if needed.
		if recursion_limit.is_some() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level) => {
					*recursion_level -= 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("Couldn't check precompile nesting").into())),
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
pub struct PrecompileSetStartingWith<A, P, C = ()> {
	precompile_set: P,
	current_recursion_level: RefCell<BTreeMap<H160, u16>>,
	_phantom: PhantomData<(A, C)>,
}

impl<A, P, C> PrecompileSetFragment for PrecompileSetStartingWith<A, P, C>
where
	A: Get<&'static [u8]>,
	P: PrecompileSet + Default,
	C: PrecompileChecks,
{
	#[inline(always)]
	fn new() -> Self {
		Self {
			precompile_set: P::default(),
			current_recursion_level: RefCell::new(BTreeMap::new()),
			_phantom: PhantomData,
		}
	}

	#[inline(always)]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		let code_address = handle.code_address();

		if !self.is_precompile(code_address) {
			return None;
		}

		// Check DELEGATECALL config.
		let allow_delegate_call = C::supports_delegate_call().unwrap_or(false);
		if !allow_delegate_call && code_address != handle.context().address {
			return Some(Err(revert(
				"Cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		// Check and increase recursion level if needed.
		let recursion_limit = C::recursion_limit().unwrap_or(Some(0));
		if let Some(max_recursion_level) = recursion_limit {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level_map) => {
					let recursion_level = recursion_level_map.entry(code_address).or_insert(0);

					if *recursion_level > max_recursion_level {
						return Some(Err(revert("Precompile is called with too high nesting")));
					}

					*recursion_level += 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("Couldn't check precompile nesting"))),
			}
		}

		let allow_subcalls = C::allow_subcalls().unwrap_or(false);
		let mut handle = RestrictiveHandle {
			handle,
			allow_subcalls,
		};

		let res = self.precompile_set.execute(&mut handle);

		// Decrease recursion level if needed.
		if recursion_limit.is_some() {
			match self.current_recursion_level.try_borrow_mut() {
				Ok(mut recursion_level_map) => {
					let recursion_level = match recursion_level_map.get_mut(&code_address) {
						Some(recursion_level) => recursion_level,
						None => return Some(Err(revert("Couldn't retreive precompile nesting"))),
					};

					*recursion_level -= 1;
				}
				// We don't hold the borrow and are in single-threaded code, thus we should
				// not be able to fail borrowing in nested calls.
				Err(_) => return Some(Err(revert("Couldn't check precompile nesting"))),
			}
		}

		res
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

/// Wraps a precompileset fragment into a range, and will skip processing it if the address
/// is out of the range.
pub struct PrecompilesInRangeInclusive<R, P> {
	inner: P,
	range: RangeInclusive<H160>,
	_phantom: PhantomData<R>,
}

impl<S, E, P> PrecompileSetFragment for PrecompilesInRangeInclusive<(S, E), P>
where
	S: Get<H160>,
	E: Get<H160>,
	P: PrecompileSetFragment,
{
	fn new() -> Self {
		Self {
			inner: P::new(),
			range: RangeInclusive::new(S::get(), E::get()),
			_phantom: PhantomData,
		}
	}

	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		if self.range.contains(&handle.code_address()) {
			self.inner.execute(handle)
		} else {
			None
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		if self.range.contains(&address) {
			self.inner.is_precompile(address)
		} else {
			false
		}
	}

	fn used_addresses(&self) -> Vec<H160> {
		self.inner.used_addresses()
	}
}

/// Wraps a tuple of `PrecompileSetFragment` to make a real `PrecompileSet`.
pub struct PrecompileSetBuilder<R, P> {
	inner: P,
	_phantom: PhantomData<R>,
}

impl<R, P: PrecompileSetFragment> PrecompileSet for PrecompileSetBuilder<R, P> {
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		self.inner.execute(handle)
	}

	fn is_precompile(&self, address: H160) -> bool {
		self.inner.is_precompile(address)
	}
}

impl<R: pallet_evm::Config, P: PrecompileSetFragment> PrecompileSetBuilder<R, P> {
	/// Create a new instance of the PrecompileSet.
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
