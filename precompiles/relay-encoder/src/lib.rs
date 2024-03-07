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

use cumulus_primitives_core::relay_chain;

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	ensure,
	traits::ConstU32,
};
use pallet_staking::RewardDestination;
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_runtime::{traits::Dispatchable, AccountId32, Perbill};
use sp_std::vec::Vec;
use sp_std::{convert::TryInto, marker::PhantomData};
use xcm_primitives::{AvailableStakeCalls, HrmpAvailableCalls, HrmpEncodeCall, StakeEncodeCall};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_relay_runtime;
#[cfg(test)]
mod tests;

pub const REWARD_DESTINATION_SIZE_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 512;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;
type GetRewardDestinationSizeLimit = ConstU32<REWARD_DESTINATION_SIZE_LIMIT>;

/// A precompile to provide relay stake calls encoding through evm
pub struct RelayEncoderPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RelayEncoderPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + pallet_xcm_transactor::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
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

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::Bond(relay_amount, reward_destination),
		)
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
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::BondExtra(relay_amount),
		)
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

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::Unbond(relay_amount),
		)
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
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::WithdrawUnbonded(slashes),
		)
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
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::Validate(pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			}),
		)
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
		let nominated: Vec<AccountId32> = nominees
			.iter()
			.map(|&add| {
				let as_bytes: [u8; 32] = add.into();
				as_bytes.into()
			})
			.collect();
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::Nominate(nominated),
		)
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

		let encoded =
			pallet_xcm_transactor::Pallet::<Runtime>::encode_call(AvailableStakeCalls::Chill)
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

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::SetPayee(reward_destination),
		)
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

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::SetController,
		)
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
		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
			AvailableStakeCalls::Rebond(relay_amount),
		)
		.as_slice()
		.into();

		Ok(encoded)
	}
	#[precompile::public("encodeHrmpInitOpenChannel(uint32,uint32,uint32)")]
	#[precompile::public("encode_hrmp_init_open_channel(uint32,uint32,uint32)")]
	#[precompile::view]
	fn encode_hrmp_init_open_channel(
		handle: &mut impl PrecompileHandle,
		recipient: u32,
		max_capacity: u32,
		max_message_size: u32,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::hrmp_encode_call(
			HrmpAvailableCalls::InitOpenChannel(recipient.into(), max_capacity, max_message_size),
		)
		.map_err(|_| {
			RevertReason::custom("Non-implemented hrmp encoding for transactor")
				.in_field("transactor")
		})?
		.as_slice()
		.into();
		Ok(encoded)
	}

	#[precompile::public("encodeHrmpAcceptOpenChannel(uint32)")]
	#[precompile::public("encode_hrmp_accept_open_channel(uint32)")]
	#[precompile::view]
	fn encode_hrmp_accept_open_channel(
		handle: &mut impl PrecompileHandle,
		sender: u32,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::hrmp_encode_call(
			HrmpAvailableCalls::AcceptOpenChannel(sender.into()),
		)
		.map_err(|_| {
			RevertReason::custom("Non-implemented hrmp encoding for transactor")
				.in_field("transactor")
		})?
		.as_slice()
		.into();
		Ok(encoded)
	}

	#[precompile::public("encodeHrmpCloseChannel(uint32,uint32)")]
	#[precompile::public("encode_hrmp_close_channel(uint32,uint32)")]
	#[precompile::view]
	fn encode_hrmp_close_channel(
		handle: &mut impl PrecompileHandle,
		sender: u32,
		recipient: u32,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::hrmp_encode_call(
			HrmpAvailableCalls::CloseChannel(relay_chain::HrmpChannelId {
				sender: sender.into(),
				recipient: recipient.into(),
			}),
		)
		.map_err(|_| {
			RevertReason::custom("Non-implemented hrmp encoding for transactor")
				.in_field("transactor")
		})?
		.as_slice()
		.into();
		Ok(encoded)
	}

	#[precompile::public("encodeHrmpCancelOpenRequest(uint32,uint32,uint32)")]
	#[precompile::public("encode_hrmp_cancel_open_request(uint32,uint32,uint32)")]
	#[precompile::view]
	fn encode_hrmp_cancel_open_request(
		handle: &mut impl PrecompileHandle,
		sender: u32,
		recipient: u32,
		open_requests: u32,
	) -> EvmResult<UnboundedBytes> {
		// No DB access but lot of logical stuff
		// To prevent spam, we charge an arbitrary amount of gas
		handle.record_cost(1000)?;

		let encoded = pallet_xcm_transactor::Pallet::<Runtime>::hrmp_encode_call(
			HrmpAvailableCalls::CancelOpenRequest(
				relay_chain::HrmpChannelId {
					sender: sender.into(),
					recipient: recipient.into(),
				},
				open_requests,
			),
		)
		.map_err(|_| {
			RevertReason::custom("Non-implemented hrmp encoding for transactor")
				.in_field("transactor")
		})?
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
		ensure!(
			reward_destination_bytes.len() > 0,
			RevertReason::custom("Reward destinations cannot be empty")
		);
		// For simplicity we use an EvmReader here
		let mut encoded_reward_destination =
			solidity::codec::Reader::new(&reward_destination_bytes);

		// We take the first byte
		let enum_selector = encoded_reward_destination.read_raw_bytes(1)?;
		// The firs byte selects the enum variant
		match enum_selector[0] {
			0u8 => Ok(RewardDestinationWrapper(RewardDestination::Staked)),
			1u8 => Ok(RewardDestinationWrapper(RewardDestination::Stash)),
			// Deprecated in https://github.com/paritytech/polkadot-sdk/pull/2380
			2u8 => Err(RevertReason::custom(
				"`Controller` was deprecated. Use `Account(controller)` instead.",
			)
			.into()),
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

	fn write(writer: &mut solidity::codec::Writer, value: Self) {
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
			// Deprecated in https://github.com/paritytech/polkadot-sdk/pull/2380
			#[allow(deprecated)]
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
		solidity::Codec::write(writer, encoded_bytes);
	}

	fn has_static_size() -> bool {
		false
	}

	fn signature() -> String {
		UnboundedBytes::signature()
	}
}
