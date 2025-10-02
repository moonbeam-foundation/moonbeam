// Copyright 2025 Moonbeam foundation
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

//! # Common XCM tests
//!
//! A collection of XCM tests common to all runtimes

#[macro_export]
macro_rules! generate_common_xcm_tests {
	($runtime: ident) => {
		#[cfg(test)]
		pub mod common_xcm_tests {
			use crate::common::{ExtBuilder, ALICE};
			use cumulus_primitives_core::ExecuteXcm;
			use frame_support::assert_ok;
			use frame_support::traits::fungible::Inspect;
			use frame_support::traits::EnsureOrigin;
			use moonbeam_core_primitives::{AccountId, Balance};
			use parity_scale_codec::Encode;
			use sp_weights::Weight;
			use xcm::{
				latest::{prelude::AccountKey20, Assets as XcmAssets, Xcm},
				VersionedAssets, VersionedLocation, VersionedXcm,
			};
			use $runtime::{
				xcm_config::SelfReserve, Balances, PolkadotXcm, Runtime, RuntimeEvent,
				RuntimeOrigin, System,
			};

			pub(crate) fn last_events(n: usize) -> Vec<RuntimeEvent> {
				System::events()
					.into_iter()
					.map(|e| e.event)
					.rev()
					.take(n)
					.rev()
					.collect()
			}

			#[test]
			fn claim_assets_works() {
				const INITIAL_BALANCE: Balance = 10_000_000_000_000_000_000;
				const SEND_AMOUNT: Balance = 1_000_000_000_000_000_000;

				let alice = AccountId::from(ALICE);
				let balances = vec![(alice, INITIAL_BALANCE)];

				ExtBuilder::default()
					.with_balances(balances)
					.build()
					.execute_with(|| {
						let assets = XcmAssets::from((SelfReserve::get(), SEND_AMOUNT));
						// First trap some assets.
						let trapping_program =
							Xcm::builder_unsafe().withdraw_asset(assets.clone()).build();
						// Even though assets are trapped, the extrinsic returns success.
						let origin_location =
							<Runtime as pallet_xcm::Config>::ExecuteXcmOrigin::ensure_origin(
								RuntimeOrigin::signed(alice),
							)
							.expect("qed");
						let message = Box::new(VersionedXcm::V5(trapping_program));
						let mut hash = message.using_encoded(sp_io::hashing::blake2_256);
						let message = (*message).try_into().expect("qed");
						let _ = <Runtime as pallet_xcm::Config>::XcmExecutor::prepare_and_execute(
							origin_location,
							message,
							&mut hash,
							Weight::MAX,
							Weight::MAX,
						);
						assert_eq!(
							Balances::total_balance(&alice),
							INITIAL_BALANCE - SEND_AMOUNT
						);

						// Assets were indeed trapped.
						assert!(last_events(2).iter().any(|evt| matches!(
							evt,
							RuntimeEvent::PolkadotXcm(pallet_xcm::Event::AssetsTrapped { .. })
						)));

						// Now claim them with the extrinsic.
						assert_ok!(PolkadotXcm::claim_assets(
							RuntimeOrigin::signed(alice),
							Box::new(VersionedAssets::V5(assets)),
							Box::new(VersionedLocation::V5(
								AccountKey20 {
									network: None,
									key: alice.clone().into()
								}
								.into()
							)),
						));
						// Confirm that trapped assets were claimed back
						assert_eq!(Balances::total_balance(&alice), INITIAL_BALANCE);
					});
			}
		}
	};
}
