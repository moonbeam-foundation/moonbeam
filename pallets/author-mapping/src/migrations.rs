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

use crate::{BalanceOf, Config, Event, MappingWithDeposit, NimbusLookup, Pallet, RegistrationInfo};
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade, ReservableCurrency},
	weights::Weight,
};
use nimbus_primitives::NimbusId;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;

/// Migrates MappingWithDeposit map value from RegistrationInfo to RegistrationInformation,
/// thereby adding a keys: T::Keys field to the value to support VRF keys that can be looked up
/// via NimbusId.
pub struct AddAccountIdToNimbusLookup<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for AddAccountIdToNimbusLookup<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "AddAccountIdToNimbusLookup", "running migration");

		let mut read_write_count = 0u64;
		<MappingWithDeposit<T>>::translate(|nimbus_id, registration_info: RegistrationInfo<T>| {
			read_write_count += 2u64;
			if NimbusLookup::<T>::get(&registration_info.account).is_none() {
				<NimbusLookup<T>>::insert(&registration_info.account, nimbus_id);
				Some(registration_info)
			} else {
				// revoke the additional association and return the funds
				T::DepositCurrency::unreserve(
					&registration_info.account,
					registration_info.deposit,
				);

				<Pallet<T>>::deposit_event(Event::KeysRemoved {
					nimbus_id,
					account_id: registration_info.account,
					keys: registration_info.keys,
				});
				None
			}
		});
		// return weight
		read_write_count.saturating_mul(T::DbWeight::get().read + T::DbWeight::get().write)
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		use sp_std::vec::Vec;

		let mut nimbus_set: Vec<NimbusId> = Vec::new();
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			if !nimbus_set.contains(&nimbus_id) {
				Self::set_temp_storage(
					info.account,
					&format!("MappingWithDeposit{:?}Account", nimbus_id)[..],
				);
				nimbus_set.push(nimbus_id);
			}
		}
		Ok(())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		for (nimbus_id, _) in <MappingWithDeposit<T>>::iter() {
			let old_account: T::AccountId =
				Self::get_temp_storage(&format!("MappingWithDeposit{:?}Account", nimbus_id)[..])
					.expect("qed");
			let maybe_account_of_nimbus = <NimbusLookup<T>>::get(old_account);
			assert_eq!(
				Some(nimbus_id),
				maybe_account_of_nimbus,
				"New NimbusLookup dne expected NimbusID"
			);
		}
		Ok(())
	}
}

/// Migrates MappingWithDeposit map value from RegistrationInfo to RegistrationInformation,
/// thereby adding a keys: T::Keys field to the value to support VRF keys that can be looked up
/// via NimbusId.
pub struct AddKeysToRegistrationInfo<T>(PhantomData<T>);
#[derive(Encode, Decode, PartialEq, Eq, Debug, scale_info::TypeInfo)]
struct OldRegistrationInfo<AccountId, Balance> {
	account: AccountId,
	deposit: Balance,
}
fn migrate_registration_info<T: Config>(
	nimbus_id: NimbusId,
	old: OldRegistrationInfo<T::AccountId, BalanceOf<T>>,
) -> RegistrationInfo<T> {
	RegistrationInfo {
		account: old.account,
		deposit: old.deposit,
		keys: nimbus_id.into(),
	}
}
impl<T: Config> OnRuntimeUpgrade for AddKeysToRegistrationInfo<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "AddKeysToRegistrationInfo", "running migration");

		let mut read_write_count = 0u64;
		<MappingWithDeposit<T>>::translate(
			|nimbus_id, old_registration_info: OldRegistrationInfo<T::AccountId, BalanceOf<T>>| {
				read_write_count = read_write_count.saturating_add(1u64);
				Some(migrate_registration_info(nimbus_id, old_registration_info))
			},
		);
		// return weight
		read_write_count.saturating_mul(T::DbWeight::get().read + T::DbWeight::get().write)
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		// get total deposited and account for all nimbus_keys
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			Self::set_temp_storage(
				info.account,
				&format!("MappingWithDeposit{:?}Account", nimbus_id)[..],
			);
			Self::set_temp_storage(
				info.deposit,
				&format!("MappingWithDeposit{:?}Deposit", nimbus_id)[..],
			);
		}
		Ok(())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;
		// ensure new deposit and account are the same as the old ones
		// ensure new keys are equal to nimbus_id
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			let old_account: T::AccountId =
				Self::get_temp_storage(&format!("MappingWithDeposit{:?}Account", nimbus_id)[..])
					.expect("qed");
			let new_account = info.account;
			assert_eq!(
				old_account, new_account,
				"Old Account {:?} dne New Account {:?} for NimbusID {:?}",
				old_account, new_account, nimbus_id
			);
			let old_deposit: BalanceOf<T> =
				Self::get_temp_storage(&format!("MappingWithDeposit{:?}Deposit", nimbus_id)[..])
					.expect("qed");
			let new_deposit = info.deposit;
			assert_eq!(
				old_deposit, new_deposit,
				"Old Deposit {:?} dne New Deposit {:?} for NimbusID {:?}",
				old_deposit, new_deposit, nimbus_id
			);
			let nimbus_id_as_keys: T::Keys = nimbus_id.into();
			assert_eq!(
				nimbus_id_as_keys, info.keys,
				"Old NimbusID {:?} dne New Keys {:?}",
				nimbus_id_as_keys, info.keys,
			);
		}
		Ok(())
	}
}
