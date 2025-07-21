// Copyright 2019-2025 PureStake Inc.
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

//! Test environment setup helpers for XCM tests

use crate::xcm_mock::{parachain, MockNet, ParaA, XcmTransactor};
use frame_support::{assert_ok, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
use sp_std::boxed::Box;
use sp_weights::Weight;
use xcm::latest::prelude::{AccountKey20, Location, Parachain, WeightLimit};
use xcm_primitives::DEFAULT_PROOF_SIZE;
use xcm_simulator::TestExt;

// Essential functions that are actually used

pub fn reset_test_environment() {
	MockNet::reset();
}

// Location creation helpers - these are used
pub fn para_to_para_location(dest_para: u32, account: [u8; 20]) -> Location {
	Location {
		parents: 1,
		interior: [
			Parachain(dest_para),
			AccountKey20 {
				network: None,
				key: account,
			},
		]
		.into(),
	}
}

pub fn account_key20_location(account: [u8; 20]) -> Location {
	AccountKey20 {
		network: None,
		key: account,
	}
	.into()
}

pub fn parachain_location(para_id: u32) -> Location {
	Location {
		parents: 1,
		interior: [Parachain(para_id)].into(),
	}
}

// Weight limit helpers - keep the ones that are used
pub fn standard_transfer_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(80u64, DEFAULT_PROOF_SIZE))
}

pub fn standard_heavy_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(800000u64, DEFAULT_PROOF_SIZE))
}

pub fn medium_transfer_weight() -> WeightLimit {
	WeightLimit::Limited(Weight::from_parts(40000u64, DEFAULT_PROOF_SIZE))
}

// Transactor setup helpers - these are used
pub fn setup_relay_transactor_config() {
	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			None
		));
		// Root can set fee per second
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
	});
}

// Transfer execution helper that was accidentally removed but still used
pub fn execute_transfer_to_para(
	from_account: [u8; 20],
	asset: xcm::VersionedAssets,
	dest_para: u32,
	dest_account: [u8; 20],
	weight: Option<WeightLimit>,
) {
	let dest = para_to_para_location(dest_para, dest_account);
	let (chain_part, beneficiary) =
		xcm_primitives::split_location_into_chain_part_and_beneficiary(dest).unwrap();
	let weight_limit = weight.unwrap_or(standard_transfer_weight());

	ParaA::execute_with(|| {
		assert_ok!(crate::xcm_mock::parachain::PolkadotXcm::transfer_assets(
			parachain::RuntimeOrigin::signed(parachain::AccountId::from(from_account)),
			Box::new(xcm::VersionedLocation::from(chain_part)),
			Box::new(xcm::VersionedLocation::from(beneficiary)),
			Box::new(asset),
			0,
			weight_limit
		));
	});
}

// Account funding helper that was removed but still used
pub fn fund_account_native(account: &[u8; 20], amount: u128) {
	ParaA::execute_with(|| {
		let account_id = parachain::AccountId::from(*account);
		let _ = parachain::Balances::force_set_balance(
			parachain::RuntimeOrigin::root(),
			account_id.into(),
			amount,
		);
	});
}

// Call encoding helpers - this is heavily used
pub fn encode_relay_balance_transfer_call(
	dest: crate::xcm_mock::relay_chain::AccountId,
	amount: u128,
) -> Vec<u8> {
	use frame_support::traits::PalletInfo;
	use parity_scale_codec::Encode;

	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<crate::xcm_mock::relay_chain::Runtime as frame_system::Config>::PalletInfo::index::<
			crate::xcm_mock::relay_chain::Balances,
		>()
		.unwrap() as u8;

	encoded.push(index);

	let mut call_bytes =
		pallet_balances::Call::<crate::xcm_mock::relay_chain::Runtime>::transfer_allow_death {
			dest,
			value: amount.into(),
		}
		.encode();

	encoded.append(&mut call_bytes);
	encoded
}
