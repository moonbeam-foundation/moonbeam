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

use frame_support::{
	assert_ok,
	traits::{ConstU32, PalletInfo},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
	BoundedVec,
};
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use sp_std::boxed::Box;
use xcm::latest::prelude::{
	AccountKey20, Junctions, Limited, Location, PalletInstance, Parachain, Reanchorable,
};
use xcm_executor::traits::ConvertLocation;

use crate::xcm_mock::*;
use crate::xcm_testing::helpers::*;
use parity_scale_codec::Encode;
use xcm_simulator::TestExt;

#[test]
fn transact_through_signed_multilocation() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			// Relay charges 1000 for every instruction, and we have 3, so 3000
			3000.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4000.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			WEIGHT_REF_TIME_PER_SECOND as u128,
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000004100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000004100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	let total_weight = 4000004000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			false
		));
	});

	Relay::execute_with(|| {
		assert!(RelayBalances::free_balance(&para_a_account()) == 100);

		assert!(RelayBalances::free_balance(&derived) == 0);
	});
}

#[test]
fn transact_through_signed_multilocation_custom_fee_and_weight_refund() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	ParaA::execute_with(|| {
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the relay will see instead of us
	descend_origin_multilocation
		.reanchor(&Location::parent(), &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::Account32Hash::<
		relay_chain::KusamaNetwork,
		relay_chain::AccountId,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	Relay::execute_with(|| {
		// free execution, full amount received
		assert_ok!(RelayBalances::transfer_allow_death(
			relay_chain::RuntimeOrigin::signed(RELAYALICE),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(RelayBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(RelayBalances::free_balance(&para_a_account()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let encoded = encode_relay_balance_transfer_call(para_a_account(), 100u128);

	let total_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(total_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(total_weight.into()))
			},
			true
		));
	});

	Relay::execute_with(|| {
		// 100 transferred
		assert_eq!(RelayBalances::free_balance(&para_a_account()), 100);

		// 4000009000 refunded
		assert_eq!(RelayBalances::free_balance(&derived), 4000009000);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		assert!(ParaBalances::free_balance(&derived) == 0);

		assert!(ParaBalances::free_balance(&para_a_account_20()) == 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_refund() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000009100u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000009100);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::Balances>()
			.unwrap() as u8;

	encoded.push(index);

	// Then call bytes
	let mut call_bytes = pallet_balances::Call::<parachain::Runtime>::transfer_allow_death {
		// 100 to sovereign
		dest: para_a_account_20(),
		value: 100u32.into(),
	}
	.encode();
	encoded.append(&mut call_bytes);

	let overall_weight = 4000009000u64;
	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: Some(overall_weight as u128)
			},
			encoded,
			// 4000000000 for transfer + 9000 for XCM
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: Some(Limited(overall_weight.into()))
			},
			true
		));
	});

	ParaB::execute_with(|| {
		// Check the derived account was refunded
		assert_eq!(ParaBalances::free_balance(&derived), 3823903993);

		// Check the transfer was executed
		assert_eq!(ParaBalances::free_balance(&para_a_account_20()), 100);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact {
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			// 4000000000 for transfer + 4000 for XCM
			// 1-1 to fee
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer went through
		assert!(
			ParaBalances::free_balance(&PARAALICE.into())
				== parachain_b_alice_balances_before + 100
		);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_no_proxy_fails() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let mut parachain_b_alice_balances_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		parachain_b_alice_balances_before = ParaBalances::free_balance(&PARAALICE.into())
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(PARAALICE.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer wasn't executed
		assert!(ParaBalances::free_balance(&PARAALICE.into()) == parachain_b_alice_balances_before);
	});
}

#[test]
fn transact_through_signed_multilocation_para_to_para_ethereum_proxy_succeeds() {
	reset_test_environment();
	let mut ancestry = Location::parent();

	let para_b_location = Location::new(1, [Parachain(2)]);

	let para_b_balances = Location::new(1, [Parachain(2), PalletInstance(1u8)]);

	ParaA::execute_with(|| {
		// Root can set transact info
		assert_ok!(XcmTransactor::set_transact_info(
			parachain::RuntimeOrigin::root(),
			// ParaB
			Box::new(xcm::VersionedLocation::from(para_b_location.clone())),
			// Para charges 1000 for every instruction, and we have 3, so 3
			3.into(),
			20000000000.into(),
			// 4 instructions in transact through signed
			Some(4.into())
		));
		// Root can set transact info
		assert_ok!(XcmTransactor::set_fee_per_second(
			parachain::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(para_b_balances.clone())),
			parachain::ParaTokensPerSecond::get(),
		));
		ancestry = parachain::UniversalLocation::get().into();
	});

	// Let's construct the Junction that we will append with DescendOrigin
	let signed_origin: Junctions = [AccountKey20 {
		network: None,
		key: PARAALICE,
	}]
	.into();

	let mut descend_origin_multilocation = parachain::SelfLocation::get();
	descend_origin_multilocation
		.append_with(signed_origin)
		.unwrap();

	// To convert it to what the paraB will see instead of us
	descend_origin_multilocation
		.reanchor(&para_b_location, &ancestry.interior)
		.unwrap();

	let derived = xcm_builder::HashedDescription::<
		parachain::AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>::convert_location(&descend_origin_multilocation)
	.unwrap();

	let transfer_recipient = evm_account();
	let mut transfer_recipient_balance_before = 0;
	ParaB::execute_with(|| {
		assert_ok!(ParaBalances::transfer_allow_death(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			derived.clone(),
			4000000104u128,
		));
		// derived account has all funds
		assert!(ParaBalances::free_balance(&derived) == 4000000104);
		// sovereign account has 0 funds
		assert!(ParaBalances::free_balance(&para_a_account_20()) == 0);

		transfer_recipient_balance_before = ParaBalances::free_balance(&transfer_recipient.into());

		// Add proxy ALICE  -> derived
		let _ = parachain::Proxy::add_proxy_delegate(
			&PARAALICE.into(),
			derived,
			parachain::ProxyType::Any,
			0,
		);
	});

	// Encode the call. Balances transact to para_a_account
	// First index
	let mut encoded: Vec<u8> = Vec::new();
	let index =
		<parachain::Runtime as frame_system::Config>::PalletInfo::index::<parachain::EthereumXcm>()
			.unwrap() as u8;

	encoded.push(index);

	use sp_core::U256;
	// Let's do a EVM transfer
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V2(xcm_primitives::EthereumXcmTransactionV2 {
			gas_limit: U256::from(21000),
			action: pallet_ethereum::TransactionAction::Call(transfer_recipient.into()),
			value: U256::from(100),
			input: BoundedVec::<
				u8,
				ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>
			>::try_from(vec![]).unwrap(),
			access_list: None,
		});

	// Then call bytes
	let mut call_bytes = pallet_ethereum_xcm::Call::<parachain::Runtime>::transact_through_proxy {
		transact_as: PARAALICE.into(),
		xcm_transaction: eth_tx,
	}
	.encode();
	encoded.append(&mut call_bytes);

	ParaA::execute_with(|| {
		assert_ok!(XcmTransactor::transact_through_signed(
			parachain::RuntimeOrigin::signed(PARAALICE.into()),
			Box::new(xcm::VersionedLocation::from(para_b_location)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					para_b_balances
				))),
				fee_amount: None
			},
			encoded,
			TransactWeights {
				transact_required_weight_at_most: 4000000000.into(),
				overall_weight: None
			},
			false
		));
	});

	ParaB::execute_with(|| {
		// Make sure the EVM transfer was executed
		assert!(
			ParaBalances::free_balance(&transfer_recipient.into())
				== transfer_recipient_balance_before + 100
		);
	});
}
