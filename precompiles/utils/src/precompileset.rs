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

use crate::{revert, PrecompileHandle, StatefulPrecompile, H160};
use fp_evm::{PrecompileResult, PrecompileSet};
use sp_std::{cell::RefCell, marker::PhantomData};

#[derive(Clone)]
pub struct ChainedPrecompile<P, S> {
	precompile: P,
	chain: S,
	address: H160,
	allow_delegate: bool,
	current_recursion_level: RefCell<u16>,
	max_recursion_level: Option<u16>,
}

impl<P: StatefulPrecompile, S: PrecompileSet> PrecompileSet for ChainedPrecompile<P, S> {
	#[inline]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		// Move forward the chain if this is not the correct address.
		if handle.code_address() != self.address {
			return self.chain.execute(handle);
		}

		// Check DELEGATECALL
		if !self.allow_delegate && handle.code_address() != handle.context().address {
			return Some(Err(revert(
				"cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		// Check and increase recursion level if needed.
		if let Some(max_recursion_level) = self.max_recursion_level {
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
		if let Some(max_recursion_level) = self.max_recursion_level {
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

		Some(res)
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == self.address || self.chain.is_precompile(address)
	}
}

#[derive(Clone)]
pub struct ChainedPrecompileSet<P, C> {
	precompile_set: P,
	chain: C,
	address_prefix: &'static [u8],
}

impl<P: PrecompileSet, S: PrecompileSet> PrecompileSet for ChainedPrecompileSet<P, S> {
	#[inline]
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		// Move forward the chain if this is not the correct address.
		if !handle
			.code_address()
			.as_bytes()
			.starts_with(&self.address_prefix)
		{
			self.chain.execute(handle)
		} else {
			self.precompile_set.execute(handle)
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		self.chain.is_precompile(address) || self.precompile_set.is_precompile(address)
	}
}

pub struct StatefulPrecompileWrapper<T>(PhantomData<T>);

impl<T: fp_evm::Precompile> fp_evm::Precompile for StatefulPrecompileWrapper<T> {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		T::execute(handle)
	}
}

impl<T> Clone for StatefulPrecompileWrapper<T> {
	fn clone(&self) -> Self {
		Self(PhantomData)
	}
}

pub struct BuilderParams {
	max_recursion_level: Option<u16>,
	allow_delegate: bool,
}

impl BuilderParams {
	pub fn new() -> Self {
		Self {
			max_recursion_level: Some(1),
			allow_delegate: false,
		}
	}

	pub fn with_max_recursion(mut self, max_recursion_level: Option<u16>) -> Self {
		self.max_recursion_level = max_recursion_level;
		self
	}

	pub fn allow_delegatecall(mut self) -> Self {
		self.allow_delegate = true;
		self
	}
}

pub trait PrecompileSetBuilderExt: Sized {
	fn add_stateful_precompile<P: StatefulPrecompile>(
		self,
		address: H160,
		precompile: P,
		params: BuilderParams,
	) -> ChainedPrecompile<P, Self>;

	fn add_precompile<P: fp_evm::Precompile>(
		self,
		address: H160,
		params: BuilderParams,
	) -> ChainedPrecompile<StatefulPrecompileWrapper<P>, Self> {
		self.add_stateful_precompile(address, StatefulPrecompileWrapper::<P>(PhantomData), params)
	}

	fn add_precompile_set<P: PrecompileSet>(
		self,
		address_prefix: &'static [u8],
		precompile_set: P,
	) -> ChainedPrecompileSet<P, Self>;
}

impl<T: PrecompileSet> PrecompileSetBuilderExt for T {
	fn add_stateful_precompile<P: StatefulPrecompile>(
		self,
		address: H160,
		precompile: P,
		params: BuilderParams,
	) -> ChainedPrecompile<P, Self> {
		ChainedPrecompile {
			precompile,
			chain: self,
			address,
			allow_delegate: params.allow_delegate,
			current_recursion_level: RefCell::new(0),
			max_recursion_level: params.max_recursion_level,
		}
	}

	fn add_precompile_set<P: PrecompileSet>(
		self,
		address_prefix: &'static [u8],
		precompile_set: P,
	) -> ChainedPrecompileSet<P, Self> {
		ChainedPrecompileSet {
			precompile_set,
			chain: self,
			address_prefix,
		}
	}
}
