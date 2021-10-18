// Copyright 2019-2021 PureStake Inc.
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
use xcm::v1::{AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiLocation};

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
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					MultiAsset {
						id: AssetId::Concrete(MultiLocation::parent()),
						fun: Fungibility::Fungible(100)
					},
					100u64,
					vec![0u8]
				),
				Error::<Test>::UnclaimedIndex
			);

			// Root can register
			assert_ok!(XcmTransactor::register(Origin::root(), 1u64, 1));

			// Not using the same fee asset as the destination chain, so error
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					Transactors::Relay,
					1,
					MultiAsset {
						id: AssetId::Concrete(MultiLocation::new(
							1,
							Junctions::X1(Junction::Parachain(1000))
						)),
						fun: Fungibility::Fungible(100)
					},
					100u64,
					vec![0u8]
				),
				Error::<Test>::NotAllowed
			);
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

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_derivative(
				Origin::signed(1u64),
				Transactors::Relay,
				1,
				MultiAsset {
					id: AssetId::Concrete(MultiLocation::parent()),
					fun: Fungibility::Fungible(100)
				},
				100u64,
				vec![1u8]
			));
			let expected = vec![
				crate::Event::RegisterdDerivative(1u64, 1),
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
					MultiLocation::parent(),
					1u64,
					MultiAsset {
						id: AssetId::Concrete(MultiLocation::parent()),
						fun: Fungibility::Fungible(100)
					},
					100u64,
					vec![1u8],
				),
				DispatchError::BadOrigin
			);

			// fee as destination are the same, this time it should work
			assert_ok!(XcmTransactor::transact_through_sovereign(
				Origin::root(),
				MultiLocation::parent(),
				1u64,
				MultiAsset {
					id: AssetId::Concrete(MultiLocation::parent()),
					fun: Fungibility::Fungible(100)
				},
				100u64,
				vec![1u8]
			));

			let expected = vec![crate::Event::TransactedSovereign(
				1u64,
				MultiLocation::parent(),
				vec![1u8],
			)];
			assert_eq!(events(), expected);
		})
}
