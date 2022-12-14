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
use sp_std::vec::Vec;
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

		T::DbWeight::get()
			.reads(read_write_count)
			.saturating_add(T::DbWeight::get().writes(read_write_count))
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		use sp_std::collections::btree_map::BTreeMap;

		let mut nimbus_set: Vec<NimbusId> = Vec::new();
		let mut state_map: BTreeMap<NimbusId, T::AccountId> = BTreeMap::new();
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			if !nimbus_set.contains(&nimbus_id) {
				state_map.insert(nimbus_id.clone(), info.account);
				nimbus_set.push(nimbus_id);
			}
		}
		Ok(state_map.encode())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		use sp_std::collections::btree_map::BTreeMap;

		let state_map: BTreeMap<NimbusId, T::AccountId> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		for (nimbus_id, _) in <MappingWithDeposit<T>>::iter() {
			let old_account = state_map.get(&nimbus_id).expect("qed");
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
		T::DbWeight::get()
			.reads(read_write_count)
			.saturating_add(T::DbWeight::get().writes(read_write_count))
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		use sp_std::collections::btree_map::BTreeMap;

		let mut state_map: BTreeMap<NimbusId, (T::AccountId, BalanceOf<T>)> = BTreeMap::new();

		// get total deposited and account for all nimbus_keys
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			state_map.insert(nimbus_id, (info.account, info.deposit));
		}
		Ok(state_map.encode())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		use sp_std::collections::btree_map::BTreeMap;

		let state_map: BTreeMap<NimbusId, (T::AccountId, BalanceOf<T>)> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// ensure new deposit and account are the same as the old ones
		// ensure new keys are equal to nimbus_id
		for (nimbus_id, info) in <MappingWithDeposit<T>>::iter() {
			let (old_account, old_deposit) = state_map.get(&nimbus_id).expect("qed");

			let new_account = info.account;
			assert_eq!(
				old_account.clone(),
				new_account,
				"Old Account {:?} dne New Account {:?} for NimbusID {:?}",
				old_account.clone(),
				new_account,
				nimbus_id
			);
			let new_deposit = info.deposit;
			assert_eq!(
				old_deposit.clone(),
				new_deposit,
				"Old Deposit {:?} dne New Deposit {:?} for NimbusID {:?}",
				old_deposit.clone(),
				new_deposit,
				nimbus_id
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
