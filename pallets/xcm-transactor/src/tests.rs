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
use frame_support::{assert_noop, assert_ok};
use sp_std::boxed::Box;
use xcm::latest::{Junction, Junctions, MultiLocation};
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

			let expected = vec![crate::Event::RegisterdDerivative(1u64, 1)];
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
				XcmTransactor::transact_through_derivative_multilocation(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					100u64,
					vec![0u8]
				),
				Error::<Test>::UnclaimedIndex
			);

			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
					1,
					Junctions::X1(Junction::Parachain(1000))
				))),
				0,
				0,
				0,
				1,
				0
			));

			// Not using the same fee asset as the destination chain, so error
			assert_noop!(
				XcmTransactor::transact_through_derivative_multilocation(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
						1,
						Junctions::X1(Junction::Parachain(1000))
					))),
					100u64,
					vec![0u8]
				),
				Error::<Test>::AssetIsNotReserveInDestination
			);

			// Reserve but info not present, error
			assert_noop!(
				XcmTransactor::transact_through_derivative_multilocation(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
						1,
						Junctions::X1(Junction::PalletInstance(1))
					))),
					100u64,
					vec![0u8]
				),
				Error::<Test>::TransactorInfoNotSet
			);

			// Cannot exceed the max weight
			assert_noop!(
				XcmTransactor::transact_through_derivative_multilocation(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::new(
						1,
						Junctions::X1(Junction::PalletInstance(1))
					))),
					21000000000u64,
					vec![0u8]
				),
				Error::<Test>::MaxWeightTransactReached
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
				0,
				0,
				1,
				0
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative_multilocation(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				100u64,
				vec![1u8]
			));
			let expected = vec![
				crate::Event::RegisterdDerivative(1u64, 1),
				crate::Event::TransactInfoChanged(
					MultiLocation::parent(),
					RemoteTransactInfo {
						transact_extra_weight: 0,
						fee_per_byte: 0,
						base_weight: 0,
						fee_per_weight: 1,
						metadata_size: 0,
					},
				),
				crate::Event::TransactedDerivative(
					1u64,
					MultiLocation::parent(),
					Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					1,
				),
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
				0,
				0,
				1,
				0
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				CurrencyId::OtherReserve(0),
				100u64,
				vec![1u8]
			));
			let expected = vec![
				crate::Event::RegisterdDerivative(1u64, 1),
				crate::Event::TransactInfoChanged(
					MultiLocation::parent(),
					RemoteTransactInfo {
						transact_extra_weight: 0,
						fee_per_byte: 0,
						base_weight: 0,
						fee_per_weight: 1,
						metadata_size: 0,
					},
				),
				crate::Event::TransactedDerivative(
					1u64,
					MultiLocation::parent(),
					Transactors::Relay
						.encode_call(UtilityAvailableCalls::AsDerivative(1, vec![1u8])),
					1,
				),
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
					Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
					100u64,
					vec![1u8],
				),
				DispatchError::BadOrigin
			);

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				0,
				0,
				1,
				0
			));

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1u64,
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				100u64,
				vec![1u8]
			));

			let expected = vec![
				crate::Event::TransactInfoChanged(
					MultiLocation::parent(),
					RemoteTransactInfo {
						transact_extra_weight: 0,
						fee_per_byte: 0,
						base_weight: 0,
						fee_per_weight: 1,
						metadata_size: 0,
					},
				),
				crate::Event::TransactedSovereign(1u64, MultiLocation::parent(), vec![1u8]),
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
				XcmTransactor::calculate_fee_per_weight(
					[1u8; 7].to_vec(),
					578000,
					100000000,
					125000000,
					8,
					105
				),
				12204624000
			);
		})
}
