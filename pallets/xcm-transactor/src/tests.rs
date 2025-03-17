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

use crate::mock::*;
use crate::*;
use cumulus_primitives_core::relay_chain::HrmpChannelId;
use frame_support::weights::Weight;
use frame_support::{assert_noop, assert_ok, weights::constants::WEIGHT_REF_TIME_PER_SECOND};
use sp_runtime::traits::Convert;
use sp_runtime::DispatchError;
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
				XcmTransactor::register(RuntimeOrigin::signed(1u64), 1u64, 1),
				DispatchError::BadOrigin
			);

			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

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
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::UnclaimedIndex
			);

			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// TransactInfo not yet set
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::TransactorInfoNotSet
			);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::FeePerSecondNotSet
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::new(
					1,
					[Junction::Parachain(1000)]
				))),
				1
			));

			// TransactInfo present, but the asset is not a reserve of dest
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::new(1, [Junction::Parachain(1000)])
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::AssetIsNotReserveInDestination
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// Cannot exceed the max weight
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 10001u64.into(),
						overall_weight: None
					},
					false
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
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::TransactorInfoNotSet
			);

			// Root can set transact info without extra_signed being None
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::SignedTransactNotAllowedForDestination
			);

			// Root can set transact info, with extra signed
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				15000.into(),
				Some(12000.into())
			));

			// TransactInfo present, but FeePerSecond not set
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				Error::<Test>::FeePerSecondNotSet
			);

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::new(
					1,
					[Junction::Parachain(1000)]
				))),
				1
			));

			// TransactInfo present, but the asset is not a reserve of dest
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::new(1, [Junction::Parachain(1000)])
						))),
						fee_amount: None
					},
					vec![0u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
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
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				RuntimeOrigin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: None
				},
				false
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: Location::parent(),
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
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				RuntimeOrigin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: None
				},
				false
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: Location::parent(),
					call: Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					index: 1,
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();

			// Check message doesn't contain the appendix
			assert!(!sent_message.0.contains(&SetAppendix(Xcm(vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(AllCounted(1u32)),
					beneficiary: Location {
						parents: 0,
						interior: [Junction::Parachain(100)].into()
					}
				}
			]))));
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
					RuntimeOrigin::signed(1),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					Some(1u64),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: None
					},
					vec![1u8],
					OriginKind::SovereignAccount,
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
				),
				DispatchError::BadOrigin
			);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Some(1u64),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: None
				},
				vec![1u8],
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: None
				},
				false
			));

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedSovereign {
					fee_payer: Some(1u64),
					dest: Location::parent(),
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
					1000000000.into(),
					8 * WEIGHT_REF_TIME_PER_SECOND as u128
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
				XcmTransactor::calculate_fee_per_second(319324000.into(), 38620923000),
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
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			assert_eq!(XcmTransactor::index_to_account(&1).unwrap(), 1u64);

			assert_ok!(XcmTransactor::deregister(RuntimeOrigin::root(), 1));

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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Root can remove transact info
			assert_ok!(XcmTransactor::remove_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
			));

			assert!(XcmTransactor::transact_info(Location::parent()).is_none());

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::TransactInfoRemoved {
					location: Location::parent(),
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
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// weight value not set for signed transact, fails
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				Some(Weight::MAX)
			));

			// weight should overflow
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 10064u64.into(),
						overall_weight: None
					},
					false
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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				Some(1.into())
			));

			// 10000 + 1 > 10000 (max weight permitted by dest chain)
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100000u64.into(),
						overall_weight: None
					},
					false
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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				Some(1.into())
			));

			// fee per second not set, fails
			assert_noop!(
				XcmTransactor::transact_through_signed(
					RuntimeOrigin::signed(1u64),
					Box::new(xcm::VersionedLocation::V4(Location::parent())),
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					false
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
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				Some(1.into())
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// transact info and fee per second set
			// this time it should work
			assert_ok!(XcmTransactor::transact_through_signed(
				RuntimeOrigin::signed(1u64),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: None
				},
				false
			));

			let expected = vec![
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: Some(1.into()),
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedSigned {
					fee_payer: 1u64,
					dest: Location::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
		})
}

#[test]
fn test_send_through_derivative_with_custom_weight_and_fee() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters
			assert_ok!(XcmTransactor::transact_through_derivative(
				RuntimeOrigin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				},
				false
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: Location::parent(),
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
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::SovereignAccount,
				fallback_max_weight: Some(tx_weight),
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
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Some(1u64),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				},
				false
			));

			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactedSovereign {
					fee_payer: Some(1u64),
					dest: Location::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::SovereignAccount,
				fallback_max_weight: Some(tx_weight),
				call: vec![1u8].into(),
			}));
		})
}

#[test]
fn test_transact_through_sovereign_with_fee_payer_none() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			assert_ok!(XcmTransactor::transact_through_sovereign(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				// We don't specify any fee_payer, instead we pay fees with the
				// sovereign account funds directly on the destination.
				None,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				},
				false
			));

			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactedSovereign {
					fee_payer: None,
					dest: Location::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected even if we haven't indicated a
			// fee_payer.
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::SovereignAccount,
				fallback_max_weight: Some(tx_weight),
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
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// By specifying total fee and total weight, we ensure
			// that even if the transact_info is not populated,
			// the message is forged with our parameters

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_signed(
				RuntimeOrigin::signed(1u64),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				},
				false
			));

			let expected = vec![crate::Event::TransactedSigned {
				fee_payer: 1u64,
				dest: Location::parent(),
				call: vec![1u8],
			}];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::SovereignAccount,
				fallback_max_weight: Some(tx_weight),
				call: vec![1u8].into(),
			}));
		})
}

#[test]
fn test_hrmp_manipulator_init() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::InitOpen(HrmpInitParams {
					para_id: 1u32.into(),
					proposed_max_capacity: 1,
					proposed_max_message_size: 1
				}),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: tx_weight,
				call: vec![0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0].into(),
			}));
		})
}

#[test]
fn test_hrmp_manipulator_init_v2_convert_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// Change xcm version
			CustomVersionWrapper::set_version(2);

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::InitOpen(HrmpInitParams {
					para_id: 1u32.into(),
					proposed_max_capacity: 1,
					proposed_max_message_size: 1
				}),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: tx_weight,
				call: vec![0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0].into(),
			}));

			// Check message contains the new appendix
			assert!(sent_message.0.contains(&SetAppendix(Xcm(vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(AllCounted(1)),
					beneficiary: Location {
						parents: 0,
						interior: [Junction::Parachain(100)].into()
					}
				}
			]))));
		})
}

#[test]
fn test_hrmp_manipulator_init_v3_convert_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// Change xcm version
			CustomVersionWrapper::set_version(3);

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::InitOpen(HrmpInitParams {
					para_id: 1u32.into(),
					proposed_max_capacity: 1,
					proposed_max_message_size: 1
				}),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: Some(tx_weight),
				call: vec![0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0].into(),
			}));

			// Check message contains the new appendix
			assert!(sent_message.0.contains(&SetAppendix(Xcm(vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(AllCounted(1)),
					beneficiary: Location {
						parents: 0,
						interior: [Junction::Parachain(100)].into()
					}
				}
			]))));
		})
}

#[test]
fn test_hrmp_manipulator_init_v5_convert_fails() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			// Change xcm version
			CustomVersionWrapper::set_version(5);

			assert_noop!(
				XcmTransactor::hrmp_manage(
					RuntimeOrigin::root(),
					HrmpOperation::InitOpen(HrmpInitParams {
						para_id: 1u32.into(),
						proposed_max_capacity: 1,
						proposed_max_message_size: 1
					}),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: Some(total_fee)
					},
					TransactWeights {
						transact_required_weight_at_most: tx_weight,
						overall_weight: Some(Limited(total_weight))
					}
				),
				Error::<Test>::ErrorValidating
			);
		})
}

#[test]
fn test_hrmp_max_fee_errors() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 10_000_000_000_000u128;

			assert_noop!(
				XcmTransactor::hrmp_manage(
					RuntimeOrigin::root(),
					HrmpOperation::InitOpen(HrmpInitParams {
						para_id: 1u32.into(),
						proposed_max_capacity: 1,
						proposed_max_message_size: 1
					}),
					CurrencyPayment {
						currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
							Location::parent()
						))),
						fee_amount: Some(total_fee)
					},
					TransactWeights {
						transact_required_weight_at_most: tx_weight,
						overall_weight: Some(Limited(total_weight))
					}
				),
				Error::<Test>::TooMuchFeeUsed
			);
		})
}

#[test]
fn test_hrmp_manipulator_accept() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::Accept {
					para_id: 1u32.into()
				},
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: Some(tx_weight),
				call: vec![0, 0, 1, 0, 0, 0].into(),
			}));
		})
}

#[test]
fn test_hrmp_manipulator_cancel() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;
			let channel_id = HrmpChannelId {
				sender: 1u32.into(),
				recipient: 1u32.into(),
			};
			let open_requests: u32 = 1;

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::Cancel {
					channel_id,
					open_requests
				},
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: Some(tx_weight),
				call: vec![0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0].into(),
			}));
		})
}

#[test]
fn test_hrmp_manipulator_close() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// We are gonna use a total weight of 10_100, a tx weight of 100,
			// and a total fee of 100
			let total_weight: Weight = 10_100u64.into();
			let tx_weight: Weight = 100_u64.into();
			let total_fee = 100u128;

			assert_ok!(XcmTransactor::hrmp_manage(
				RuntimeOrigin::root(),
				HrmpOperation::Close(HrmpChannelId {
					sender: 1u32.into(),
					recipient: 1u32.into()
				}),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
						Location::parent()
					))),
					fee_amount: Some(total_fee)
				},
				TransactWeights {
					transact_required_weight_at_most: tx_weight,
					overall_weight: Some(Limited(total_weight))
				}
			));

			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();
			// Lets make sure the message is as expected
			assert!(sent_message
				.0
				.contains(&WithdrawAsset((Location::here(), total_fee).into())));
			assert!(sent_message.0.contains(&BuyExecution {
				fees: (Location::here(), total_fee).into(),
				weight_limit: Limited(total_weight),
			}));
			assert!(sent_message.0.contains(&Transact {
				origin_kind: OriginKind::Native,
				fallback_max_weight: Some(tx_weight),
				call: vec![0, 0, 1, 0, 0, 0, 1, 0, 0, 0].into(),
			}));
		})
}

#[test]
fn test_transact_through_derivative_with_refund_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				RuntimeOrigin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: Some(Limited(1000.into()))
				},
				true
			));
			let expected = vec![
				crate::Event::RegisteredDerivative {
					account_id: 1u64,
					index: 1,
				},
				crate::Event::TransactInfoChanged {
					location: Location::parent(),
					remote_info: RemoteTransactInfoWithMaxWeight {
						transact_extra_weight: 0.into(),
						max_weight: 10000.into(),
						transact_extra_weight_signed: None,
					},
				},
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedDerivative {
					account_id: 1u64,
					dest: Location::parent(),
					call: Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					index: 1,
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();

			// Check message contains the new appendix
			assert!(sent_message.0.contains(&SetAppendix(Xcm(vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(AllCounted(1u32)),
					beneficiary: Location {
						parents: 0,
						interior: [Junction::Parachain(100)].into()
					}
				}
			]))));
		})
}

#[test]
fn test_transact_through_derivative_with_refund_fails_overall_weight_not_set() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Root can register
			assert_ok!(XcmTransactor::register(RuntimeOrigin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				0.into(),
				10000.into(),
				None
			));

			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// fee as destination are the same, this time it should work
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					RuntimeOrigin::signed(1u64),
					Transactors::Relay,
					1,
					CurrencyPayment {
						currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
						fee_amount: None
					},
					vec![1u8],
					TransactWeights {
						transact_required_weight_at_most: 100u64.into(),
						overall_weight: None
					},
					true
				),
				Error::<Test>::RefundNotSupportedWithTransactInfo
			);
		})
}

#[test]
fn test_transact_through_signed_with_refund_works() {
	ExtBuilder::default()
		.with_balances(vec![])
		.build()
		.execute_with(|| {
			// Set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// Overall weight to use
			let total_weight: Weight = 10_100u64.into();
			assert_ok!(XcmTransactor::transact_through_signed(
				RuntimeOrigin::signed(1u64),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				CurrencyPayment {
					currency: Currency::AsCurrencyId(CurrencyId::OtherReserve(0)),
					fee_amount: None
				},
				vec![1u8],
				TransactWeights {
					transact_required_weight_at_most: 100u64.into(),
					overall_weight: Some(Limited(total_weight))
				},
				true
			));

			let expected = vec![
				crate::Event::DestFeePerSecondChanged {
					location: Location::parent(),
					fee_per_second: 1,
				},
				crate::Event::TransactedSigned {
					fee_payer: 1u64,
					dest: Location::parent(),
					call: vec![1u8],
				},
			];
			assert_eq!(events(), expected);
			let sent_messages = mock::sent_xcm();
			let (_, sent_message) = sent_messages.first().unwrap();

			// Check message contains the new appendix
			assert!(sent_message.0.contains(&SetAppendix(Xcm(vec![
				RefundSurplus,
				DepositAsset {
					assets: Wild(AllCounted(1u32)),
					beneficiary: Location {
						parents: 0,
						interior: [
							Junction::Parachain(100),
							AccountIdToLocation::convert(1)
								.interior
								.take_first()
								.unwrap()
						]
						.into()
					}
				}
			]))));
		})
}
