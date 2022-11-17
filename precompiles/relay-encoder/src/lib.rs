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

//! Precompile to encode relay staking calls via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use cumulus_primitives_core::relay_chain;
use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	ensure,
	traits::ConstU32,
};
use pallet_staking::RewardDestination;
use precompile_utils::{data::String, prelude::*};
use sp_core::{H256, U256};
use sp_runtime::AccountId32;
use sp_runtime::Perbill;
use sp_std::vec::Vec;
use sp_std::{convert::TryInto, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_relay_runtime;
#[cfg(test)]
mod tests;

pub enum AvailableStakeCalls {
	Bond(
		relay_chain::AccountId,
		relay_chain::Balance,
		pallet_staking::RewardDestination<relay_chain::AccountId>,
	),
	BondExtra(relay_chain::Balance),
	Unbond(relay_chain::Balance),
	WithdrawUnbonded(u32),
	Validate(pallet_staking::ValidatorPrefs),
	Nominate(Vec<relay_chain::AccountId>),
	Chill,
	SetPayee(pallet_staking::RewardDestination<relay_chain::AccountId>),
	SetController(relay_chain::AccountId),
	Rebond(relay_chain::Balance),
}

pub trait StakeEncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8>;
}

pub const REWARD_DESTINATION_SIZE_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 512;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;
type GetRewardDestinationSizeLimit = ConstU32<REWARD_DESTINATION_SIZE_LIMIT>;

/// A precompile to provide relay stake calls encoding through evm
pub struct RelayEncoderPrecompile<Runtime, RelayRuntime>(PhantomData<(Runtime, RelayRuntime)>);

#[precompile_utils::precompile]
impl<Runtime, RelayRuntime> RelayEncoderPrecompile<Runtime, RelayRuntime>
where
	RelayRuntime: StakeEncodeCall,
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	#[precompile::public("encodeBond(uint256,uint256,bytes)")]
	#[precompile::public("encode_bond(uint256,uint256,bytes)")]
	#[precompile::view]
	fn encode_bond(
		handle: &mut impl PrecompileHandle,
		controller_address: U256,
		amount: U256,
		reward_destination: RewardDestinationWrapper,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let address: [u8; 32] = controller_address.into();
		let relay_amount = u256_to_relay_amount(amount)?;
		let reward_destination = reward_destination.into();

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Bond(
			address.into(),
			relay_amount,
			reward_destination,
		))
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
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::BondExtra(relay_amount))
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
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let relay_amount = u256_to_relay_amount(amount)?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Unbond(relay_amount))
			.as_slice()
			.into();

		Ok(encoded)
	}

	#[precompile::public("encodeWithdrawUnbonded(uint32)")]
	#[precompile::public("encode_withdraw_unbonded(uint32)")]
	#[precompile::view]
	fn encode_withdraw_unbonded(
		handle: &mut impl PrecompileHandle,
		slashes: u32,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::WithdrawUnbonded(slashes))
			.as_slice()
			.into();

		Ok(encoded)
	}

	#[precompile::public("encodeValidate(uint256,bool)")]
	#[precompile::public("encode_validate(uint256,bool)")]
	#[precompile::view]
	fn encode_validate(
		handle: &mut impl PrecompileHandle,
		comission: SolidityConvert<U256, u32>,
		blocked: bool,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let fraction = Perbill::from_parts(comission.converted());
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Validate(
			pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			},
		))
		.as_slice()
		.into();

		Ok(encoded)
	}

	#[precompile::public("encodeNominate(uint256[])")]
	#[precompile::public("encode_nominate(uint256[])")]
	#[precompile::view]
	fn encode_nominate(
		handle: &mut impl PrecompileHandle,
		nominees: BoundedVec<U256, GetArrayLimit>,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let nominees: Vec<_> = nominees.into();
		let nominated: Vec<AccountId32> = nominees
			.iter()
			.map(|&add| {
				let as_bytes: [u8; 32] = add.into();
				as_bytes.into()
			})
			.collect();
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Nominate(nominated))
			.as_slice()
			.into();

		Ok(encoded)
	}

	#[precompile::public("encodeChill()")]
	#[precompile::public("encode_chill()")]
	#[precompile::view]
	fn encode_chill(handle: &mut impl PrecompileHandle) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Chill)
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
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let reward_destination = reward_destination.into();

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::SetPayee(reward_destination))
			.as_slice()
			.into();

		Ok(encoded)
	}

	#[precompile::public("encodeSetController(uint256)")]
	#[precompile::public("encode_set_controller(uint256)")]
	#[precompile::view]
	fn encode_set_controller(
		handle: &mut impl PrecompileHandle,
		controller: U256,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let controller: [u8; 32] = controller.into();

		let encoded =
			RelayRuntime::encode_call(AvailableStakeCalls::SetController(controller.into()))
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
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Rebond(relay_amount))
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

// A wrapper to be able to implement here the EvmData reader
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

impl EvmData for RewardDestinationWrapper {
	fn read(reader: &mut EvmDataReader) -> MayRevert<Self> {
		let reward_destination = reader.read::<BoundedBytes<GetRewardDestinationSizeLimit>>()?;
		let reward_destination_bytes: Vec<_> = reward_destination.into();
		ensure!(
			reward_destination_bytes.len() > 0,
			RevertReason::custom("Reward destinations cannot be empty")
		);
		// For simplicity we use an EvmReader here
		let mut encoded_reward_destination = EvmDataReader::new(&reward_destination_bytes);

		// We take the first byte
		let enum_selector = encoded_reward_destination.read_raw_bytes(1)?;
		// The firs byte selects the enum variant
		match enum_selector[0] {
			0u8 => Ok(RewardDestinationWrapper(RewardDestination::Staked)),
			1u8 => Ok(RewardDestinationWrapper(RewardDestination::Stash)),
			2u8 => Ok(RewardDestinationWrapper(RewardDestination::Controller)),
			3u8 => {
				let address = encoded_reward_destination.read::<H256>()?;
				Ok(RewardDestinationWrapper(RewardDestination::Account(
					address.as_fixed_bytes().clone().into(),
				)))
			}
			4u8 => Ok(RewardDestinationWrapper(RewardDestination::None)),
			_ => Err(RevertReason::custom("Unknown reward destination").into()),
		}
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut encoded: Vec<u8> = Vec::new();
		let encoded_bytes: UnboundedBytes = match value.0 {
			RewardDestination::Staked => {
				encoded.push(0);
				encoded.as_slice().into()
			}
			RewardDestination::Stash => {
				encoded.push(1);
				encoded.as_slice().into()
			}
			RewardDestination::Controller => {
				encoded.push(2);
				encoded.as_slice().into()
			}
			RewardDestination::Account(address) => {
				encoded.push(3);
				let address_bytes: [u8; 32] = address.into();
				encoded.append(&mut address_bytes.to_vec());
				encoded.as_slice().into()
			}
			RewardDestination::None => {
				encoded.push(4);
				encoded.as_slice().into()
			}
		};
		EvmData::write(writer, encoded_bytes);
	}

	fn has_static_size() -> bool {
		false
	}

	fn solidity_type() -> String {
		UnboundedBytes::solidity_type()
	}
}
