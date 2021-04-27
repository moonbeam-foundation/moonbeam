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

//! Precompile to call parachain-staking runtime methods via the EVM
use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::Precompile;
use sp_core::H160;
use sp_core::U256;
use sp_std::convert::TryFrom;
use sp_std::convert::TryInto;
use sp_std::fmt::Debug;
use sp_std::{marker::PhantomData, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from parachain_staking.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		log::info!("In parachain staking wrapper");

		// Basic sanity checking for length
		// https://solidity-by-example.org/primitives/

		const SELECTOR_SIZE_BYTES: usize = 4;

		if input.len() < 4 {
			return Err(ExitError::Other("input length less than 4 bytes".into()));
		}

		log::info!("Made it past preliminary length check");
		log::info!("context.caller is {:?}", context.caller);

		// Write first accessor this hacky way. Make it nicer later.
		if input[0..SELECTOR_SIZE_BYTES] == [0x8e, 0x50, 0x80, 0xe7] {
			// This selector means we are in the is_nominator precompile.
			log::info!("Matched accessor selector");

			// parse the address

			// TODO generalize Amar's parsing functions below
			// For some reason there are 12 blank bytes. Not sure why.
			let nominator = H160::from_slice(&input[16..36]);

			log::info!("Checking whether {:?} is a nominator", nominator);

			// fetch data from pallet
			let is_nominator =
				parachain_staking::Pallet::<Runtime>::is_nominator(&nominator.into());

			log::info!("Result from pallet is {:?}", is_nominator);

			// Solidity's bool type is 256 bits as shown by these examples https://docs.soliditylang.org/en/v0.8.0/abi-spec.html
			// But I admit the comparison to `uint8` is a little confusing.
			let mut result_bytes = [0u8; 32];
			if is_nominator {
				result_bytes[31] = 1;
			}

			log::info!("Result bytes are {:?}", result_bytes);

			//TODO figure out how much gas it costs to check whether you're a nominator.
			// That function will not naturally be benchmarked because it is not dispatchable
			// I guess the heavy part will be one storage read.
			let gas_consumed = 0;

			return Ok((ExitSucceed::Returned, result_bytes.to_vec(), gas_consumed));
		}

		// Parse the function selector
		let inner_call = match input[0..SELECTOR_SIZE_BYTES] {
			[0xad, 0x76, 0xed, 0x5a] => Self::join_candidates(&input[SELECTOR_SIZE_BYTES..])?,
			[0xb7, 0x69, 0x42, 0x19] => Self::leave_candidates()?,
			[0x76, 0x7e, 0x04, 0x50] => Self::go_offline()?,
			[0xd2, 0xf7, 0x3c, 0xeb] => Self::go_online()?,
			[0x28, 0x9b, 0x6b, 0xa7] => Self::candidate_bond_less(&input[SELECTOR_SIZE_BYTES..])?,
			[0xc5, 0x7b, 0xd3, 0xa8] => Self::candidate_bond_more(&input[SELECTOR_SIZE_BYTES..])?,
			[0x82, 0xf2, 0xc8, 0xdf] => Self::nominate(&input[SELECTOR_SIZE_BYTES..])?,
			[0xe8, 0xd6, 0x8a, 0x37] => Self::leave_nominators()?,
			[0x4b, 0x65, 0xc3, 0x4b] => Self::revoke_nomination(&input[SELECTOR_SIZE_BYTES..])?,
			[0xf6, 0xa5, 0x25, 0x69] => Self::nominator_bond_less(&input[SELECTOR_SIZE_BYTES..])?,
			[0x97, 0x1d, 0x44, 0xc8] => Self::nominator_bond_more(&input[SELECTOR_SIZE_BYTES..])?,
			_ => {
				log::info!("Failed to match function selector in staking wrapper precompile");
				return Err(ExitError::Other(
					"No staking wrapper method at selector given selector".into(),
				));
			}
		};

		let outer_call: Runtime::Call = inner_call.into();
		let info = outer_call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let valid_weight = info.weight <= Runtime::GasWeightMapping::gas_to_weight(gas_limit);
			if !valid_weight {
				return Err(ExitError::OutOfGas);
			}
		}
		log::info!("Made it past gas check");

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		log::info!("Gonna call with origin {:?}", origin);

		match outer_call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let gas_used = Runtime::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);
				//TODO Should this be returned?
				Ok((ExitSucceed::Stopped, Default::default(), gas_used))
			}
			Err(e) => {
				log::info!("Parachain staking call via evm failed {:?}", e);
				Err(ExitError::Other(
					"Parachain staking call via EVM failed".into(),
				))
			}
		}
	}
}

fn form_nominator_args<Runtime>(input: &[u8]) -> Result<(H160, BalanceOf<Runtime>), ExitError>
where
	Runtime: parachain_staking::Config,
	BalanceOf<Runtime>: TryFrom<U256>,
{
	const COLLATOR_SIZE_BYTES: usize = 20;
	const AMOUNT_SIZE_BYTES: usize = 32;
	const TOTAL_SIZE_BYTES: usize = COLLATOR_SIZE_BYTES + AMOUNT_SIZE_BYTES;

	if input.len() != TOTAL_SIZE_BYTES {
		log::info!(
			"Aborting because input length was invalid. Got {} bytes, expected {}",
			input.len(),
			TOTAL_SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for nominator call arguments".into(),
		));
	}

	let collator = H160::from_slice(&input[0..COLLATOR_SIZE_BYTES]);

	let amount: BalanceOf<Runtime> =
		U256::from_big_endian(&input[COLLATOR_SIZE_BYTES..TOTAL_SIZE_BYTES])
			.try_into()
			.map_err(|_| {
				ExitError::Other("amount is too large for Runtime's balance type".into())
			})?;
	Ok((collator, amount))
}

fn form_collator_args<Runtime>(input: &[u8]) -> Result<BalanceOf<Runtime>, ExitError>
where
	Runtime: parachain_staking::Config,
	BalanceOf<Runtime>: TryFrom<U256>,
{
	const AMOUNT_SIZE_BYTES: usize = 32;

	if input.len() != AMOUNT_SIZE_BYTES {
		log::info!(
			"Aborting because input length was invalid. Got {} bytes, expected {}",
			input.len(),
			AMOUNT_SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for collator call arguments".into(),
		));
	}

	let amount: BalanceOf<Runtime> = U256::from_big_endian(&input[0..AMOUNT_SIZE_BYTES])
		.try_into()
		.map_err(|_| ExitError::Other("amount is too large for Runtime's balance type".into()))?;
	Ok(amount)
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn join_candidates(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = form_collator_args::<Runtime>(input)?;

		log::info!("Collator stake amount is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::join_candidates(amount))
	}

	fn leave_candidates() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::leave_candidates())
	}

	fn go_offline() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::go_offline())
	}

	fn go_online() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::go_online())
	}

	fn candidate_bond_more(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = form_collator_args::<Runtime>(input)?;

		log::info!("Collator bond increment is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::candidate_bond_more(
			amount,
		))
	}

	fn candidate_bond_less(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = form_collator_args::<Runtime>(input)?;

		log::info!("Collator bond decrement is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::candidate_bond_less(
			amount,
		))
	}

	fn nominate(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let (collator, amount) = form_nominator_args::<Runtime>(input)?;

		log::info!("Collator account is {:?}", collator);
		log::info!("Nomination amount is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::nominate(
			collator.into(),
			amount,
		))
	}

	fn leave_nominators() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::leave_nominators())
	}

	fn revoke_nomination(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		const COLLATOR_SIZE_BYTES: usize = 20;

		if input.len() != COLLATOR_SIZE_BYTES {
			log::info!(
				"Aborting because input length was invalid. Got {} bytes, expected {}",
				input.len(),
				COLLATOR_SIZE_BYTES,
			);
			return Err(ExitError::Other(
				"Incorrect input length for revoke nomination call argument".into(),
			));
		}

		let collator = H160::from_slice(&input[0..COLLATOR_SIZE_BYTES]);

		log::info!("Collator account is {:?}", collator);

		Ok(parachain_staking::Call::<Runtime>::revoke_nomination(
			collator.into(),
		))
	}

	fn nominator_bond_more(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let (collator, amount) = form_nominator_args::<Runtime>(input)?;

		log::info!("Collator account is {:?}", collator);
		log::info!("Nomination increment is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::nominator_bond_more(
			collator.into(),
			amount,
		))
	}

	fn nominator_bond_less(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let (collator, amount) = form_nominator_args::<Runtime>(input)?;

		log::info!("Collator account is {:?}", collator);
		log::info!("Nomination decrement is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::nominator_bond_less(
			collator.into(),
			amount,
		))
	}
}
