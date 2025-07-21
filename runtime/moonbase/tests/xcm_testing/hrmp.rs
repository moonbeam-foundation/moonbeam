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

use crate::xcm_mock::*;
use frame_support::assert_ok;
use pallet_xcm_transactor::{
	Currency, CurrencyPayment, HrmpInitParams, HrmpOperation, TransactWeights,
};
use sp_std::boxed::Box;
use xcm::latest::prelude::{Limited, Location};
use xcm_simulator::TestExt;

use cumulus_primitives_core::relay_chain::HrmpChannelId;

#[test]
fn hrmp_init_accept_through_root() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_b_account(),
			1000u128
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp init channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::InitOpen(HrmpInitParams {
				para_id: 2u32.into(),
				proposed_max_capacity: 1,
				proposed_max_message_size: 1
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelRequested {
				sender: 1u32.into(),
				recipient: 2u32.into(),
				proposed_max_capacity: 1u32,
				proposed_max_message_size: 1u32,
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
	ParaB::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		// Root can send hrmp accept channel
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Accept {
				para_id: 1u32.into()
			},
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});

	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::OpenChannelAccepted {
				sender: 1u32.into(),
				recipient: 2u32.into(),
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}

#[test]
fn hrmp_close_works() {
	MockNet::reset();

	Relay::execute_with(|| {
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			para_a_account(),
			1000u128
		));
		assert_ok!(Hrmp::force_open_hrmp_channel(
			relay_chain::RuntimeOrigin::root(),
			1u32.into(),
			2u32.into(),
			1u32,
			1u32
		));
		assert_ok!(Hrmp::force_process_hrmp_open(
			relay_chain::RuntimeOrigin::root(),
			1u32
		));
	});

	ParaA::execute_with(|| {
		let total_fee = 1_000u128;
		let total_weight: u64 = 1_000_000_000;
		let tx_weight: u64 = 500_000_000;
		assert_ok!(XcmTransactor::hrmp_manage(
			parachain::RuntimeOrigin::root(),
			HrmpOperation::Close(HrmpChannelId {
				sender: 1u32.into(),
				recipient: 2u32.into()
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_fee)
			},
			TransactWeights {
				transact_required_weight_at_most: tx_weight.into(),
				overall_weight: Some(Limited(total_weight.into()))
			}
		));
	});
	Relay::execute_with(|| {
		let expected_event: relay_chain::RuntimeEvent =
			polkadot_runtime_parachains::hrmp::Event::ChannelClosed {
				by_parachain: 1u32.into(),
				channel_id: HrmpChannelId {
					sender: 1u32.into(),
					recipient: 2u32.into(),
				},
			}
			.into();
		assert!(relay_chain::relay_events().contains(&expected_event));
	});
}
