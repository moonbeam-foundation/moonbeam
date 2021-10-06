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

			assert_eq!(XcmTransactor::claimed_indices(&1).unwrap(), 1u64);
			assert_eq!(XcmTransactor::account_to_index(&1u64).unwrap(), 1);

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
					MultiLocation::parent(),
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

			// Not using the same fee as the destination chain, so error
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					MultiLocation::parent(),
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

			// SelfLocation not working
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					MultiLocation::new(1, Junctions::X1(Junction::Parachain(100))),
					1,
					MultiAsset {
						id: AssetId::Concrete(MultiLocation::new(
							1,
							Junctions::X1(Junction::Parachain(100))
						)),
						fun: Fungibility::Fungible(100)
					},
					100u64,
					vec![0u8]
				),
				Error::<Test>::NotCrossChainTransfer
			);

			// Invalid Destination. Transact should go to a consensus system, not an account/pallet index itself
			assert_noop!(
				XcmTransactor::transact_through_derivative(
					Origin::signed(1u64),
					MultiLocation::new(
						1,
						Junctions::X2(Junction::Parachain(100), Junction::GeneralIndex(1))
					),
					1,
					MultiAsset {
						id: AssetId::Concrete(MultiLocation::new(
							1,
							Junctions::X1(Junction::Parachain(100))
						)),
						fun: Fungibility::Fungible(100)
					},
					100u64,
					vec![0u8]
				),
				Error::<Test>::InvalidDest
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
				MultiLocation::parent(),
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
				crate::Event::Transacted(
					1u64,
					MultiLocation::parent(),
					UtilityCallEncoder::encode_call(UtilityAvailableCalls::AsDerivative(
						1,
						vec![1u8],
					)),
				),
			];
			assert_eq!(events(), expected);
		})
}
