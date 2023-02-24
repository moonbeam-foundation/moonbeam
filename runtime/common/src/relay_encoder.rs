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

// Relay encoder impl that uses relay indices stored in pallet-xcm-transactor
use parity_scale_codec::Encode;
use sp_std::vec::Vec;

// trait that gets the (pallet, call) index
// TODO: extract function that takes input call: *AvailableCalls and outputs (pallet_index, call_index)

pub struct CommonEncoder<R>(pub sp_std::marker::PhantomData<R>);
impl<R: pallet_xcm_transactor::Config> xcm_primitives::UtilityEncodeCall for CommonEncoder<R> {
	fn encode_call(self, call: xcm_primitives::UtilityAvailableCalls) -> Vec<u8> {
		match call {
			xcm_primitives::UtilityAvailableCalls::AsDerivative(a, b) => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(
					pallet_xcm_transactor::RelayPalletIndices::<R>::get()
						.unwrap()
						.pallets
						.utility,
				);
				// call index
				encoded_call.push(
					pallet_xcm_transactor::RelayPalletIndices::<R>::get()
						.unwrap()
						.calls
						.utility
						.as_derivative,
				);
				// encoded argument
				encoded_call.append(&mut a.encode());
				encoded_call.append(&mut b.clone());
				encoded_call
			}
		}
	}
} // TODO: add test to verify this works

// impl xcm_primitives::HrmpEncodeCall for CommonEncoder {
// 	fn hrmp_encode_call(
// 		call: xcm_primitives::HrmpAvailableCalls,
// 	) -> Result<Vec<u8>, xcm::latest::Error> {
// 		match call {
// 			xcm_primitives::HrmpAvailableCalls::InitOpenChannel(a, b, c) => Ok(RelayCall::Hrmp(
// 				HrmpCall::InitOpenChannel(a.clone(), b.clone(), c.clone()),
// 			)
// 			.encode()),
// 			xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(a) => {
// 				Ok(RelayCall::Hrmp(HrmpCall::AcceptOpenChannel(a.clone())).encode())
// 			}
// 			xcm_primitives::HrmpAvailableCalls::CloseChannel(a) => {
// 				Ok(RelayCall::Hrmp(HrmpCall::CloseChannel(a.clone())).encode())
// 			}
// 		}
// 	}
// }

impl<R: pallet_xcm_transactor::Config> pallet_evm_precompile_relay_encoder::StakeEncodeCall
	for CommonEncoder<R>
{
	fn encode_call(call: pallet_evm_precompile_relay_encoder::AvailableStakeCalls) -> Vec<u8> {
		match call {
			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Bond(a, b, c) => {
				todo!()
				//RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::BondExtra(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Unbond(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::WithdrawUnbonded(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Validate(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Chill => {
				let mut encoded_call: Vec<u8> = Vec::new();
				// pallet index
				encoded_call.push(
					pallet_xcm_transactor::RelayPalletIndices::<R>::get()
						.unwrap()
						.pallets
						.staking,
				);
				let mut staking_call_index = pallet_xcm_transactor::RelayPalletIndices::<R>::get()
					.unwrap()
					.calls
					.staking
					.chill
					.to_le_bytes()
					.to_vec();
				staking_call_index.pop();
				// call index
				encoded_call.append(&mut staking_call_index);
				encoded_call
				//RelayCall::Stake(StakeCall::Chill).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetPayee(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetController(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Rebond(a) => {
				todo!()
				//RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Nominate(a) => {
				todo!()
				// let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
				// 	a.iter().map(|add| (*add).clone().into()).collect();

				// RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}
