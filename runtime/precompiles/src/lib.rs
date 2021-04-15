// Copyright 2019-2021 PureStake Inc.
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

#![cfg_attr(not(feature = "std"), no_std)]

use std::u64;

use codec::Decode;
use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use num::BigUint;
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::{Config, Precompile, PrecompileSet};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use sp_core::offchain::Duration;
use sp_core::H160;
use sp_core::U256;
use sp_io::offchain;
use sp_std::convert::TryFrom;
use sp_std::convert::TryInto;
use sp_std::{marker::PhantomData, vec::Vec};
use std::fmt::Debug;

/// A precompile intended to burn gas and/or time without actually doing any work.
/// Meant for testing.
///
/// Expects call data to include two u64 values:
/// 1) The gas to sacrifice (charge)
/// 2) The time in msec to sleep
/// TODO: use feature flags / somehow prevent this from deployment onto live networks (or not?)
struct Sacrifice;

impl Precompile for Sacrifice {
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		_context: &Context,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		const INPUT_SIZE_BYTES: usize = 16;

		// input should be exactly 16 bytes (two 8-byte unsigned ints in big endian)
		if input.len() != INPUT_SIZE_BYTES {
			return Err(ExitError::Other(
				"input length for Sacrifice must be exactly 16 bytes".into(),
			));
		}

		// create 8-byte buffers and populate them from calldata...
		let mut gas_cost_buf: [u8; 8] = [0; 8];
		let mut msec_cost_buf: [u8; 8] = [0; 8];

		gas_cost_buf.copy_from_slice(&input[0..8]);
		msec_cost_buf.copy_from_slice(&input[8..16]);

		// then read them into a u64 as big-endian...
		let gas_cost = u64::from_be_bytes(gas_cost_buf);
		let msec_cost = u64::from_be_bytes(msec_cost_buf);

		// ensure we can afford our sacrifice...
		if let Some(gas_left) = target_gas {
			if gas_left < gas_cost {
				return Err(ExitError::OutOfGas);
			}
		}

		// TODO: impose gas-per-second constraint?

		if msec_cost > 0 {
			// TODO: log statement here
			let deadline = offchain::timestamp();
			deadline.add(Duration::from_millis(msec_cost));
			offchain::sleep_until(deadline);
		}

		// TODO Should this actually be stopped?
		// https://ethervm.io/#F3
		// REvisit: does solidity void contract issue stop or return
		Ok((ExitSucceed::Returned, [0u8; 0].to_vec(), gas_cost))
	}
}

/// A precompile to wrap the functionality from parachain_staking::nominate.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct NominateWrapper<Runtime> {
	_phantom_data: PhantomData<(Runtime)>,
}

// type BalanceOf<Runtime> =
// 	<<Runtime as parachain_staking::Config>::Currency as Currency<Runtime::AccountId>>::Balance;

impl<Runtime> Precompile for NominateWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	<<Runtime as parachain_staking::Config>::Currency as Currency<Runtime::AccountId>>::Balance:
		TryFrom<U256> + Debug,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		// Basic sanity checking for length
		// https://solidity-by-example.org/primitives/

		// Params are:
		// Collator target (size 20 bytes, address)
		// Nomination Amoutn: (size 32 bytes, U256 (for now))
		const COLLATOR_SIZE_BYTES: usize = 20;
		const AMOUNT_SIZE_BYTES: usize = 32;
		if input.len() != COLLATOR_SIZE_BYTES + AMOUNT_SIZE_BYTES {
			return Err(ExitError::Other(
				"input length for Sacrifice must be exactly 16 bytes".into(),
			));
		}

		// Parse input into Rust types
		let mut collator_buf: [u8; COLLATOR_SIZE_BYTES] = [0; COLLATOR_SIZE_BYTES];
		let mut amount_buf: [u8; AMOUNT_SIZE_BYTES] = [0; AMOUNT_SIZE_BYTES];

		// TODO is this copy necessary? Probably.
		collator_buf.copy_from_slice(&input[0..COLLATOR_SIZE_BYTES]);
		amount_buf
			.copy_from_slice(&input[COLLATOR_SIZE_BYTES..COLLATOR_SIZE_BYTES + AMOUNT_SIZE_BYTES]);

		// Convert to right data types
		let collator = H160::from_slice(&collator_buf);

		// TODO handle the error for too-big numbers
		let amount: <<Runtime as parachain_staking::Config>::Currency as Currency<
			Runtime::AccountId,
		>>::Balance = sp_core::U256::from_big_endian(&amount_buf)
			.try_into()
			.map_err(|_| {
				ExitError::Other("amount is too large for Runtime's balance type".into())
			})?;

		println!("Collator account is {:?}", collator);
		println!("Amount is {:?}", amount);

		// Construct a call
		let inner_call = parachain_staking::Call::<Runtime>::nominate(collator.into(), amount);

		let outer_call: <Runtime as frame_system::Config>::Call = inner_call.into();

		let info = outer_call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let valid_weight = info.weight <= Runtime::GasWeightMapping::gas_to_weight(gas_limit);
			if !valid_weight {
				return Err(ExitError::OutOfGas);
			}
		}

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		match outer_call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let gas_used = Runtime::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);
				Ok((ExitSucceed::Stopped, Default::default(), gas_used))
			}
			Err(_) => Err(ExitError::Other(
				"Parachain staking nomination failed".into(),
			)),
		}
	}
}

/// The PrecompileSet installed in the Moonbeam runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct MoonbeamPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for MoonbeamPrecompiles<R>
where
	R: Config,
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context)),
			// Moonbeam precompiles :
			a if a == hash(255) => Some(Dispatch::<R>::execute(input, target_gas, context)),
			// Moonbeam testing-only precompile(s):
			a if a == hash(511) => Some(Sacrifice::execute(input, target_gas, context)),
			_ => None,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::{Duration, Instant};
	// use sp_io::TestExternalities; XXX
	extern crate hex;

	/*
	 * XXX
	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}
	*/

	#[test]
	fn test_invalid_input_length() -> std::result::Result<(), ExitError> {
		let cost: u64 = 1;

		// TODO: this is very not-DRY, it would be nice to have a default / test impl in Frontier
		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		// should fail with input of 15 byte length
		let input: [u8; 15] = [0; 15];
		assert_eq!(
			Sacrifice::execute(&input, Some(cost), &context),
			Err(ExitError::Other(
				"input length for Sacrifice must be exactly 16 bytes".into()
			)),
		);

		// should fail with input of 17 byte length
		let input: [u8; 17] = [0; 17];
		assert_eq!(
			Sacrifice::execute(&input, Some(cost), &context),
			Err(ExitError::Other(
				"input length for Sacrifice must be exactly 16 bytes".into()
			)),
		);

		Ok(())
	}

	#[test]
	fn test_gas_consumption() -> std::result::Result<(), ExitError> {
		let mut input: [u8; 16] = [0; 16];
		input[..8].copy_from_slice(&123456_u64.to_be_bytes());

		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		assert_eq!(
			Sacrifice::execute(&input, None, &context),
			Ok((ExitSucceed::Returned, [0u8; 0].to_vec(), 123456)),
		);

		Ok(())
	}

	#[test]
	fn test_oog() -> std::result::Result<(), ExitError> {
		let mut input: [u8; 16] = [0; 16];
		input[..8].copy_from_slice(&100_u64.to_be_bytes());

		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		assert_eq!(
			Sacrifice::execute(&input, Some(99), &context),
			Err(ExitError::OutOfGas),
		);

		Ok(())
	}

	/*
	 * TODO: the sleep() function inside the precompile must be run within an externalities
	 *       environment
	#[test]
	fn test_sleep() -> std::result::Result<(), ExitError> {

		new_test_ext().execute_with(|| {
			let mut input: [u8; 16] = [0; 16];
			input[8..].copy_from_slice(&10_u64.to_be_bytes()); // should be 10ms

			let context: Context = Context {
				address: Default::default(),
				caller: Default::default(),
				apparent_value: From::from(0),
			};

			let start = Instant::now();

			assert_eq!(
				Sacrifice::execute(&input, Some(99), &context),
				Ok((ExitSucceed::Returned, [0u8; 0].to_vec(), 0)),
			);

			assert!(start.elapsed().as_millis() > 10);
			assert!(start.elapsed().as_millis() < 20); // give plenty of room, but put some bound on it

			Ok(())
		});
	}
	*/
}
