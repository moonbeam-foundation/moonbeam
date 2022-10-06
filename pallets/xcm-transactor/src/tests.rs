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

use crate::mock::*;
use crate::*;
use frame_support::dispatch::DispatchError;
use frame_support::{
	assert_noop, assert_ok, storage::migration::put_storage_value,
	weights::constants::WEIGHT_PER_SECOND, Blake2_128Concat,
};
use sp_std::boxed::Box;
use xcm::latest::prelude::*;
use xcm_primitives::{UtilityAvailableCalls, UtilityEncodeCall};
#[test]
fn test_register_address() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Only root can do this, as specified in runtime
			assert_noop!(
				XcmTransactor::register(Origin::signed(1u64), 1u64, 1),
				DispatchError::BadOrigin
			);

			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			assert_eq!(XcmTransactor::index_to_account(&1).unwrap(), 1u64);

			let expected = vec![crate::Event::RegisteredDerivative {
				account_id: 1u64,
				index: 1,
			}];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_transact_through_derivative_errors() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Non-claimed index so cannot transfer
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::UnclaimedIndex
			);

			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// TransactInfo not yet set
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::TransactorInfoNotSet
			);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::FeePerSecondNotSet
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
					1,
					Junctions::X1(Junction::Parachain(1000))
				))),
				1
			));

			// TransactInfo present, but the asset is not a reserve of dest
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::new(
								1,
								Junctions::X1(Junction::Parachain(1000))
							))
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::AssetIsNotReserveInDestination
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// Cannot exceed the max weight
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 10001u64,
						overall_weight: None
					}
				),
				Error::<Test>::MaxWeightTransactReached
			);
		})
}

#[test]
fn test_transact_through_signed_errors() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// TransactInfo not yet set
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::TransactorInfoNotSet
			);

			// Root can set transact info without extra_signed being None
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::SignedTransactNotAllowedForDestination
			);

			// Root can set transact info, with extra signed
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				15000,
				Some(12000)
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::FeePerSecondNotSet
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
					1,
					Junctions::X1(Junction::Parachain(1000))
				))),
				1
			));

			// TransactInfo present, but the asset is not a reserve of dest
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::new(
								1,
								Junctions::X1(Junction::Parachain(1000))
							))
						)),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::AssetIsNotReserveInDestination
			);
		})
}

#[test]
fn test_transact_through_derivative_multilocation_success() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
						MultiLocation::parent()
					))),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64,
					overall_weight: None
				}
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactInfoChanged {
					location: MultiLocation::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0,
						max_weight: 10000,
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: MultiLocation::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: MultiLocation::parent(),
					call: Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					index: 1,
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_transact_through_derivative_success() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64,
					overall_weight: None
				}
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactInfoChanged {
					location: MultiLocation::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0,
						max_weight: 10000,
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: MultiLocation::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: MultiLocation::parent(),
					call: Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					index: 1,
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_root_can_transact_through_sovereign() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Only root can do this
			assert_noop!(
				XcmTransactor::transact_through_sovereign(
					Origin::signed(1),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					1u64,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(
							xcm::VersionedMultiLocation::V1(MultiLocation::parent())
						)),
						fee_amount: None
					},
					vec![1u8],
					OriginKind::SovereignAccount,
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				DispatchError::BadOrigin
			);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1u64,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
						MultiLocation::parent()
					))),
					fee_amount: None
				},
				vec![1u8],
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 100u64,
					overall_weight: None
				}
			));

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: MultiLocation::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0,
						max_weight: 10000,
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: MultiLocation::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedSovereign {
					fee_payer: 1u64,
					dest: MultiLocation::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_fee_calculation_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			assert_eq!(
				XcmTransactor::calculate_fee_per_second(
					1000000000,
					8 * WEIGHT_PER_SECOND.ref_time() as u128
				),
				8000000000
			);
		})
}

// Kusama case
#[test]
fn test_fee_calculation_works_kusama_0_9_20_case() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// 38620923000 * 319324000/1e12 = 12332587.6161
			// integer arithmetic would round this to 12332587
			// we test here that it rounds up to 12332588 instead
			assert_eq!(
				XcmTransactor::calculate_fee_per_second(319324000, 38620923000),
				12332588
			);
		})
}

#[test]
fn de_registering_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			assert_eq!(XcmTransactor::index_to_account(&1).unwrap(), 1u64);

			assert_ok!(XcmTransactor::deregister(Origin::root(), 1));

			assert!(XcmTransactor::index_to_account(&1).is_none());

			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::DeRegisteredDerivative { index: 1 },
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn removing_transact_info_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// Root can remove transact info
			assert_ok!(XcmTransactor::remove_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
			));

			assert!(XcmTransactor::transact_info(MultiLocation::parent()).is_none());

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: MultiLocation::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0,
						max_weight: 10000,
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::TransactInfoRemoved {
					location: MultiLocation::parent(),
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_transact_through_signed_fails_if_transact_info_not_set_at_all() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// fee as destination are the same, this time it should work
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::TransactorInfoNotSet
			);
		})
}

#[test]
fn test_transact_through_signed_fails_if_weight_is_not_set() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				None
			));

			// weight value not set for signed transact, fails
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::SignedTransactNotAllowedForDestination
			);
		})
}

#[test]
fn test_transact_through_signed_fails_if_weight_overflows() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				Some(u64::MAX)
			));

			// weight should overflow
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 10064,
						overall_weight: None
					}
				),
				Error::<Test>::WeightOverflow
			);
		})
}

#[test]
fn test_transact_through_signed_fails_if_weight_is_bigger_than_max_weight() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				Some(1)
			));

			// 10000 + 1 > 10000 (max weight permitted by dest chain)
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100000u64,
						overall_weight: None
					}
				),
				Error::<Test>::MaxWeightTransactReached
			);
		})
}

#[test]
fn test_transact_through_signed_fails_if_fee_per_second_not_set() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				Some(1)
			));

			// fee per second not set, fails
			assert_noop!(
				XcmTransactor::transact_through_signed(
					Origin::signed(1u64),
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64,
						overall_weight: None
					}
				),
				Error::<Test>::FeePerSecondNotSet
			);
		})
}

#[test]
fn test_transact_through_signed_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000,
				Some(1)
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// transact info and fee per second set
			// this time it should work
			assert_ok!(XcmTransactor::transact_through_signed(
				Origin::signed(1u64),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64,
					overall_weight: None
				}
			));

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: MultiLocation::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0,
						max_weight: 10000,
						transact_extra_weight_signed: Some(1),
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: MultiLocation::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedSigned {
					fee_payer: 1u64,
					dest: MultiLocation::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_signed_weight_and_fee_per_second_migration_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			let pallet_prefix: &[u8] = b"XcmTransactor";
			let storage_item_prefix: &[u8] = b"TransactInfoWithWeightLimit";
			use frame_support::traits::OnRuntimeUpgrade;
			use frame_support::StorageHasher;
			use parity_scale_codec::Encode;

			// This is the previous struct, which we have moved to migrations
			let old_transact_info_with_fee_per_sec =
				migrations::OldRemoteTransactInfoWithFeePerSecond {
					transact_extra_weight: 1,
					fee_per_second: 2,
					max_weight: 3,
				};
			// This is the new struct
			let expected_transacted_info = RemoteTransactInfoWithMaxWeight {
				transact_extra_weight: 1,
				max_weight: 3,
				transact_extra_weight_signed: None,
			};
			// This is the new struct
			let expected_destination_fee_per_second = 2u128;

			// We populate the previous key with the previous struct
			put_storage_value(
				pallet_prefix,
				storage_item_prefix,
				&Blake2_128Concat::hash(&MultiLocation::parent().encode()),
				old_transact_info_with_fee_per_sec,
			);
			// We run the migration
			crate::migrations::TransactSignedWeightAndFeePerSecond::<Test>::on_runtime_upgrade();

			// We make sure that the new storage key is populated
			assert_eq!(
				XcmTransactor::transact_info(MultiLocation::parent()).unwrap(),
				expected_transacted_info,
			);
			// We make sure that the new storage key is populated
			assert_eq!(
				XcmTransactor::dest_asset_fee_per_second(MultiLocation::parent()).unwrap(),
				expected_destination_fee_per_second,
			);
		})
}

#[test]
fn test_send_through_derivative_with_custom_weight_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight = 10_100u64;
			let tx_weight = 100_u64;
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters
			assert_ok!(XcmTransactor::transact_through_derivative(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(total_weight)
				}
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: MultiLocation::parent(),
					call: Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					index: 1,
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((MultiLocation::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (MultiLocation::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: tx_weight,
				call: Transactors::Relay
					.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8]))
					.into(),
			}));
		})
}

#[test]
fn test_send_through_sovereign_with_custom_weight_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight = 10_100u64;
			let tx_weight = 100_u64;
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1u64,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
						MultiLocation::parent()
					))),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(total_weight)
				}
			));

			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactedSovereign {
					fee_payer: 1u64,
					dest: MultiLocation::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((MultiLocation::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (MultiLocation::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: tx_weight,
				call: vec![1u8].into(),
			}));
		})
}

#[test]
fn test_send_through_signed_with_custom_weight_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight = 10_100u64;
			let tx_weight = 100_u64;
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_signed(
				Origin::signed(1u64),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
						MultiLocation::parent()
					))),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(total_weight)
				}
			));

			let expected = vec![crate::Event::TransactedSigned {
				fee_payer: 1u64,
				dest: MultiLocation::parent(),
				call: vec![1u8],
			}];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((MultiLocation::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (MultiLocation::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: tx_weight,
				call: vec![1u8].into(),
			}));
		})
}
