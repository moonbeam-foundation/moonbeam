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
			use frame_support::weights::{constants::WEIGHT_REF_TIME_PER_SECOND, WeightToFee as _};
			use moonbeam_core_primitives::{AccountId, Balance};
			use pallet_xcm_weight_trader::{SupportedAssets, RELATIVE_PRICE_DECIMALS};
			use parity_scale_codec::Encode;
			use sp_weights::Weight;
			use xcm::latest::Location;
			use xcm::{
				latest::{prelude::AccountKey20, Assets as XcmAssets, Xcm},
				VersionedAssets, VersionedLocation, VersionedXcm,
			};
			use $runtime::{
				xcm_config::SelfReserve, Balances, PolkadotXcm, Runtime, RuntimeEvent,
				RuntimeOrigin, System, XcmTransactor, XcmWeightTrader,
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
			fn dest_asset_fee_per_second_matches_configured_fee_not_relative_price() {
				fn set_fee_per_second_for_location(
					location: Location,
					fee_per_second: u128,
				) -> Result<(), ()> {
					let native_amount_per_second: u128 =
						<Runtime as pallet_xcm_weight_trader::Config>::WeightToFee::weight_to_fee(
							&Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, 0),
						)
						.try_into()
						.map_err(|_| ())?;
					let precision_factor = 10u128.pow(RELATIVE_PRICE_DECIMALS);
					let relative_price: u128 = if fee_per_second > 0u128 {
						native_amount_per_second
							.saturating_mul(precision_factor)
							.saturating_div(fee_per_second)
					} else {
						0u128
					};
					if SupportedAssets::<Runtime>::contains_key(&location) {
						let enabled = SupportedAssets::<Runtime>::get(&location).ok_or(())?.0;
						SupportedAssets::<Runtime>::insert(&location, (enabled, relative_price));
					} else {
						SupportedAssets::<Runtime>::insert(&location, (true, relative_price));
					}
					Ok(())
				}

				ExtBuilder::default().build().execute_with(|| {
					// Scenario: the reserve asset is 5x more valuable than the native asset.
					// The actual fee-per-second on the reserve chain is native_fee_per_second / 5.
					let native_fee_per_second = WEIGHT_REF_TIME_PER_SECOND as u128;
					let actual_fee_per_second = native_fee_per_second
						.checked_div(5)
						.expect("division by 5 should not overflow");

					let location = Location::parent();

					// Configure weight-trader storage using a helper that writes the relative price.
					set_fee_per_second_for_location(location.clone(), actual_fee_per_second)
						.expect("must be able to configure fee per second");

					// dest_asset_fee_per_second must return the true fee-per-second that callers
					// expect.
					let reported = XcmTransactor::dest_asset_fee_per_second(&location)
						.expect("fee should be set");

					assert_eq!(reported, actual_fee_per_second);
				});
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
