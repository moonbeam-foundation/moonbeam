use frame_support::traits::{
	fungible::{self, NativeOrWithId},
	tokens::Pay,
};
use moonbeam_core_primitives::{AssetId, Balance};
use pallet_moonbeam_foreign_assets::AssetsById;
use sp_core::U256;
use sp_runtime::DispatchError;

pub struct MultiAssetPaymaster<R>(sp_std::marker::PhantomData<R>);
impl<R> Pay for MultiAssetPaymaster<R>
where
	R: frame_system::Config
		+ pallet_treasury::Config
		+ pallet_balances::Config
		+ pallet_asset_manager::Config
		+ pallet_assets::Config
		+ pallet_moonbeam_foreign_assets::Config,
{
	type Balance = Balance;
	type Beneficiary = R::AccountId;
	type AssetKind = NativeOrWithId<AssetId>;
	type Id = ();
	type Error = DispatchError;
	fn pay(
		who: &Self::Beneficiary,
		asset_kind: Self::AssetKind,
		amount: Self::Balance,
	) -> Result<Self::Id, Self::Error> {
		match asset_kind {
			Self::AssetKind::Native => {
				// Pay account with native balance
				<pallet_balances::Pallet<R> as fungible::Mutate<_>>::transfer(
					&pallet_treasury::Pallet::<R>::account_id(),
					who,
					<R as pallet_balances::Config>::Balance::try_from(amount)
						.map_err(|_| pallet_treasury::Error::<R>::PayoutError)?,
					frame_support::traits::tokens::Preservation::Expendable,
				)?;
				Ok(())
			}
			Self::AssetKind::WithId(id) => {
				// Check in the foreign assets first
				if let Some(_asset_loc) = AssetsById::<R>::get(id) {
					// Pay if asset found
					pallet_moonbeam_foreign_assets::Pallet::<R>::transfer(
						id,
						pallet_treasury::Pallet::<R>::account_id(),
						who.clone(),
						U256::from(amount as u128),
					)
					.map_err(|_| pallet_treasury::Error::<R>::PayoutError)?;
					return Ok(());
				}
				Err(pallet_moonbeam_foreign_assets::Error::<R>::AssetDoesNotExist.into())
			}
		}
	}

	fn check_payment(_id: Self::Id) -> frame_support::traits::tokens::PaymentStatus {
		frame_support::traits::tokens::PaymentStatus::Success
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(_: &Self::Beneficiary, asset: Self::AssetKind, amount: Self::Balance) {
		use xcm::opaque::v4::Junction::Parachain;
		use xcm::v4::Location;
		let treasury = pallet_treasury::Pallet::<R>::account_id();
		match asset {
			Self::AssetKind::Native => {
				pallet_balances::Pallet::<R>::mint_into(
					&treasury,
					<R as pallet_balances::Config>::Balance::try_from(amount)
						.map_err(|_| pallet_treasury::Error::<R>::PayoutError)
						.unwrap(),
				);
			}
			Self::AssetKind::WithId(id) => {
				// Check if asset exists & create if required
				if let None = AssetsById::<R>::get(id) {
					let location = Location::new(1, [Parachain(1000)]);
					pallet_moonbeam_foreign_assets::Pallet::<R>::do_create_asset(
						id,
						location,
						18,
						"DEV".as_bytes().to_vec().try_into().expect("too long"),
						"DEV".as_bytes().to_vec().try_into().expect("too long"),
						None,
					)
					.expect("failed to create asset");
				}
				// Fund treasury account
				pallet_moonbeam_foreign_assets::Pallet::<R>::mint_into(
					id,
					treasury,
					U256::from(amount as u128),
				)
				.expect("failed to mint asset into treasury account");
			}
		}
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_concluded(_: Self::Id) {}
}
