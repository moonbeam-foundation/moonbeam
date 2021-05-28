//TODO license

//! State migrations for the Moonbase runtime

use frame_support::{traits::{Get, OnRuntimeUpgrade, ReservableCurrency}, weights::Weight};
use crate::{Balance, Runtime, AuthorMapping};

/// The type responsible for migrating the moonbase runtime from ersion to version. Each migration
/// is gated by runtime spec version, so only the correct migrations should be supplied. If you
/// need to migrate storage, add the migration logic here.
pub struct MoonbaseMigrationStation;

impl OnRuntimeUpgrade for MoonbaseMigrationStation {
	fn on_runtime_upgrade() -> Weight {

		log::info!("🗺️ Running MoonbaseMigrationStation");

		let previous_version: u32 = frame_system::LastRuntimeUpgrade::<Runtime>::get().map(|rv| rv.spec_version.into()).unwrap_or(0);

		log::info!("🗺️ Previous version: {:?}", previous_version);
		log::info!("🗺️ Upgrade version:  {:?}", crate::VERSION.spec_version);

		match previous_version {
			v if v < 40 => panic!("Migrating from runtimes below spec_version 40 is not supported!"),
			v if v < 43 => migrate_author_mapping_storage_deposit_to_proper_units(),
			_ => panic!("Cannot migrate from future runtime versions"),

		}
	}
}

fn migrate_author_mapping_storage_deposit_to_proper_units() -> Weight {
	
	// The previous version of the runtime had a security deposit of 100 wei.
	// The new runtime uses 100 Units (with 18 decimals). To migrate, we iterate each
	// registartion and reserve the additional required deposit. If an account does not have
	// enough to cover the additional deposit, their registration is removed and their
	// original deposit is refunded.

	log::info!("🗺️ In helper");

	let old_deposit: Balance = 100u32.into();
	let additional_deposit = <Runtime as pallet_author_mapping::Config>::DepositAmount::get() - old_deposit;

	// Iterate the entire storage mapping
	pallet_author_mapping::Mapping::<Runtime>::translate(|_, account_id| {
		// Try to reserve the additional amount
		match <Runtime as pallet_author_mapping::Config>::DepositCurrency::reserve(&account_id, additional_deposit) {
			// If they can afford it, return the same account_id to preserve the mapping
			Ok(()) => {
				log::info!("🗺️ Reserving more for account: {:?}", account_id);
				Some(account_id)
			},
			// If they can't afford the additional deposit
			Err(_) => {
				log::info!("🗺️ Account can't afford deposit: {:?}", account_id);

				// Refund the original deposit.
				<Runtime as pallet_author_mapping::Config>::DepositCurrency::unreserve(&account_id, old_deposit);

				// Return None to remove the registration from the map
				None
			}
		}
	});

	log::info!("🗺️ about to return from helper");

	//TODO No idea what weight I should be returning.
	10_000u32.into()
}