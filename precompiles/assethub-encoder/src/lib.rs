// Copyright 2025 Moonbeam foundation
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

//! Precompile to encode AssetHub staking calls via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_primitives_core::relay_chain;

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use pallet_staking::RewardDestination;
use parity_scale_codec::{Decode, Encode};
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_runtime::{traits::Dispatchable, AccountId32, Perbill};
use sp_std::vec::Vec;
use sp_std::{convert::TryInto, marker::PhantomData};
use xcm_primitives::{AssetHubTransactor, AvailableStakeCalls};

pub const REWARD_DESTINATION_SIZE_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 512;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;
type GetRewardDestinationSizeLimit = ConstU32<REWARD_DESTINATION_SIZE_LIMIT>;

/// A precompile to provide AssetHub stake calls encoding through evm
pub struct AssetHubEncoderPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> AssetHubEncoderPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + pallet_xcm_transactor::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as pallet_xcm_transactor::Config>::Transactor: AssetHubTransactor,
{
	#[precompile::public("encodeBond(uint256,bytes)")]
	#[precompile::public("encode_bond(uint256,bytes)")]
	#[precompile::view]
	fn encode_bond(
		handle: &mut impl PrecompileHandle,
		amount: U256,
		reward_destination: RewardDestinationWrapper,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let reward_destination = reward_destination.into();

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Bond(relay_amount, reward_destination),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeBondExtra(uint256)")]
	#[precompile::public("encode_bond_extra(uint256)")]
	#[precompile::view]
	fn encode_bond_extra(
		handle: &mut impl PrecompileHandle,
		amount: U256,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::BondExtra(relay_amount),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeUnbond(uint256)")]
	#[precompile::public("encode_unbond(uint256)")]
	#[precompile::view]
	fn encode_unbond(
		handle: &mut impl PrecompileHandle,
		amount: U256,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Unbond(relay_amount),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeWithdrawUnbonded(uint32)")]
	#[precompile::public("encode_withdraw_unbonded(uint32)")]
	#[precompile::view]
	fn encode_withdraw_unbonded(
		handle: &mut impl PrecompileHandle,
		num_slashing_spans: u32,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::WithdrawUnbonded(num_slashing_spans),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeValidate(uint256,bool)")]
	#[precompile::public("encode_validate(uint256,bool)")]
	#[precompile::view]
	fn encode_validate(
		handle: &mut impl PrecompileHandle,
		commission: Convert<U256, u32>,
		blocked: bool,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let fraction = Perbill::from_parts(commission.converted());

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Validate(pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			}),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeNominate(bytes32[])")]
	#[precompile::public("encode_nominate(bytes32[])")]
	#[precompile::view]
	fn encode_nominate(
		handle: &mut impl PrecompileHandle,
		nominees: BoundedVec<H256, GetArrayLimit>,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let nominees: Vec<_> = nominees.into();
		let nominees_as_account_ids: Vec<AccountId32> = nominees
			.iter()
			.map(|&add| {
				let as_bytes: [u8; 32] = add.into();
				as_bytes.into()
			})
			.collect();

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Nominate(nominees_as_account_ids),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeChill()")]
	#[precompile::public("encode_chill()")]
	#[precompile::view]
	fn encode_chill(handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Chill,
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeSetPayee(bytes)")]
	#[precompile::public("encode_set_payee(bytes)")]
	#[precompile::view]
	fn encode_set_payee(
		handle: &mut impl PrecompileHandle,
		reward_destination: RewardDestinationWrapper,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let reward_destination = reward_destination.into();

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::SetPayee(reward_destination),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeSetController()")]
	#[precompile::public("encode_set_controller()")]
	#[precompile::view]
	fn encode_set_controller(handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::SetController,
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeRebond(uint256)")]
	#[precompile::public("encode_rebond(uint256)")]
	#[precompile::view]
	fn encode_rebond(
		handle: &mut impl PrecompileHandle,
		amount: U256,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_stake_call(
			&<Runtime as pallet_xcm_transactor::Config>::Transactor::asset_hub(),
			AvailableStakeCalls::Rebond(relay_amount),
		)
		.map_err(|_| revert("Transactor not configured"))?
		.as_slice()
		.into();

		Ok(encoded)
	}
}

pub fn u256_to_relay_amount(value: U256) -> EvmResult<relay_chain::Balance> {
	value
		.try_into()
		.map_err(|_| revert("amount is too large for provided balance type"))
}

// A wrapper to be able to implement here the solidity::Codec reader
#[derive(Clone, Eq, PartialEq)]
pub struct RewardDestinationWrapper(RewardDestination<AccountId32>);

impl From<RewardDestination<AccountId32>> for RewardDestinationWrapper {
	fn from(reward_dest: RewardDestination<AccountId32>) -> Self {
		RewardDestinationWrapper(reward_dest)
	}
}

impl Into<RewardDestination<AccountId32>> for RewardDestinationWrapper {
	fn into(self) -> RewardDestination<AccountId32> {
		self.0
	}
}

impl solidity::Codec for RewardDestinationWrapper {
	fn read(reader: &mut solidity::codec::Reader) -> MayRevert<Self> {
		let reward_destination = reader.read::<BoundedBytes<GetRewardDestinationSizeLimit>>()?;
		let reward_destination_bytes: Vec<_> = reward_destination.into();
		if reward_destination_bytes.is_empty() {
			return Err(RevertReason::custom(
				"Error while decoding reward destination: input too short",
			)
			.into());
		}
		let reward_destination =
			RewardDestination::<AccountId32>::decode(&mut reward_destination_bytes.as_slice())
				.map_err(|_| RevertReason::custom("Error while decoding reward destination"))?;

		Ok(reward_destination.into())
	}

	fn write(writer: &mut solidity::codec::Writer, value: Self) {
		let encoded = value.0.encode();
		BoundedBytes::<GetRewardDestinationSizeLimit>::write(writer, encoded.as_slice().into());
	}

	fn has_static_size() -> bool {
		false
	}

	fn signature() -> String {
		BoundedBytes::<GetRewardDestinationSizeLimit>::signature()
	}
}

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_assethub_runtime;
#[cfg(test)]
mod tests;
