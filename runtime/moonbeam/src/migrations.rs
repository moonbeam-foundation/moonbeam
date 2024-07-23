// Copyright 2024 Moonbeam Foundation Inc.
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

//! # Moonbeam specific Migrations

use crate::Runtime;
use frame_support::{storage::unhashed::{get_raw, put_raw}, weights::Weight, StorageHasher, ensure};
use frame_system::AccountInfo;
use pallet_balances::AccountData;
use pallet_migrations::{GetMigrations, Migration};
use sp_core::bytes::from_hex;
use sp_std::{prelude::*, vec};
use sp_core::hex2array;
use moonbeam_core_primitives::{Balance, Index};
use parity_scale_codec::{Decode, Encode};

pub struct MoonbeamMigrations;

impl GetMigrations for MoonbeamMigrations {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		vec![
			// Runtime 3001
			// Box::new(PalletStakingMultiplyRoundLenBy2)
			// Runtime 3100
			Box::new(PalletReferendaRestoreDeposits),
		]
	}
}

// The corrupted referenda is an array of tuples containing:
// 1. the referenda index
// 2. the referenda.referendumInfoFor.key
// 3. the corrupted state
// 4. the correct state
const CORRUPTED_REFERENDA: &'static [(i32, &str, &str, &str)] = &[
	(
		5, // referenda index
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	969e061847da7e84337ea78dc577cd1d05000000",
		"0x04f5d541000101e79d1889bbd6cd8d348c5c9ad3254e92ee14e60e0000a0dec5adc9353600000000000000",
		"0x04f5d5410001e79d1889bbd6cd8d348c5c9ad3254e92ee14e60e0000a0dec5adc935360000000000000000",
	),
	(
		6,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	484d257daa10da0e6fd9b5529818625c06000000",
		"0x04fcd541000101381d106c440f92654827d2b2c637dd5b38a362a70000a0dec5adc9353600000000000000",
		"0x04fcd5410001381d106c440f92654827d2b2c637dd5b38a362a70000a0dec5adc935360000000000000000",
	),
	(
		8,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	be1f3931028cc05c2e18a319e8f64f9e08000000",
		"0x01a8ed41000000",
		"0x01a8ed410001e79d1889bbd6cd8d348c5c9ad3254e92ee14e60e0000a0dec5adc935360000000000000000",
	),
	(
		9,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	f71f22775221b1945fe6cfa3c6550c7c09000000",
		"0x0209d841000101e79d1889bbd6cd8d348c5c9ad3254e92ee14e60e0000a0dec5adc9353600000000000000",
		"0x0209d8410001e79d1889bbd6cd8d348c5c9ad3254e92ee14e60e0000a0dec5adc935360000000000000000",
	),
	(
		13,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	7e19bf46066d74446ee15c8c2715f7030d000000",
		"0x041485440001018405e969da8b4be4ab07bb712735483806f135f60000a0dec5adc9353600000000000000",
		"0x0414854400018405e969da8b4be4ab07bb712735483806f135f60000a0dec5adc935360000000000000000",
	),
	(
		14,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	4671f3b0f75141e3511b3597f3223e920e000000",
		"0x01c0a444000000",
		"0x01c0a44400018405e969da8b4be4ab07bb712735483806f135f60000a0dec5adc935360000000000000000",
	),
	(
		20,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	264c24258678941962c0f6292ecb4cad14000000",
		"0x019d5047000000",
		"0x019d504700011b35a6e8cfdc01bcfc089b06a629db296aeeb2680000a0dec5adc935360000000000000000",
	),
	(
		21,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	218d19ff794c68b9ffc65f2a631525f415000000",
		"0x02b8a248000101ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc9353600000000000000",
		"0x02b8a2480001ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc935360000000000000000",
	),
	(
		22,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	ad1bdd11ebe8658f0084ba66824e9fe616000000",
		"0x045ac548000101739c7a8f29367feb823ac1d3ac3028cc1f0ab6390000a0dec5adc9353600000000000000",
		"0x045ac5480001739c7a8f29367feb823ac1d3ac3028cc1f0ab6390000a0dec5adc935360000000000000000",
	),
	(
		23,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	e9687713e18188d35299dc9fb723898417000000",
		"0x0356d248000101ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc9353600000000000000",
		"0x0356d2480001ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc935360000000000000000",
	),
	(
		24,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	b8a8eb3050f394da75435ac79b58aa4b18000000",
		"0x01228149000000",
		"0x012281490001739c7a8f29367feb823ac1d3ac3028cc1f0ab6390000a0dec5adc935360000000000000000",
	),
	(
		26,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	9ca40f4d7dc2de6440f9998e6fe95a091a000000",
		"0x0112df47000000",
		"0x0112df47000158ce55a4efd27f8fe836a1bcdd2bc7f0a81fa6270000a0dec5adc935360000000000000000",
	),
	(
		27,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	81a37b0404f99b797427fa89b4d07c721b000000",
		"0x01a6f149000000",
		"0x01a6f1490001932a2f7b446367db8284403ba5879020cce3635a0000a0dec5adc935360000000000000000",
	),
	(
		30,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	28210239547ef1b1a87a0b1c87bd80bc1e000000",
		"0x0168c74b000000",
		"0x0168c74b00017b9a3a0bd813c61006d94c7206137052f1cd6ce10000a0dec5adc935360000000000000000",
	),
	(
		32,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	7fbfc6805b5d0ea54a7de3ea8df2f8da20000000",
		"0x01c2ca4c000000",
		"0x01c2ca4c00017ba99e99bc669b3508aff9cc0a898e869459f8770000a0dec5adc935360000000000000000",
	),
	(
		37,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	ea9c3d391651517623fdb56677aed5a825000000",
		"0x02847051000101bc7ce80a47ccc70eedb84267e9f0b9678f5a0c8b0000a0dec5adc9353600000000000000",
		"0x028470510001bc7ce80a47ccc70eedb84267e9f0b9678f5a0c8b0000a0dec5adc935360000000000000000",
	),
	(
		38,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	2c399c11d782416ac5a2034d728fe62826000000",
		"0x01ae4f53000000",
		"0x01ae4f530001ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc935360000000000000001\
	44624edd30abdc5aad29f5b233e6e610e954054f000040b2bac9e0191e02000000000000",
	),
	(
		40,
		"0x0f6738a0ee80c8e74cd2c7417c1e25569613e9bbc07e304aa9a1af9b85898e5a\
	73fecc050cc370e6185a23480f7479b128000000",
		"0x01b9d253000000",
		"0x01b9d2530001ee27bbebce601c882d6778a3352ffff95289493f0000a0dec5adc935360000000000000001\
	06d2ab1ed0c25b0629d277afd6fd928d232d41b2000040b2bac9e0191e02000000000000",
	),
];

const INVALID_STORAGE_KEY: &[u8] = b"invalid_storage_key";
const INVALID_VALUE_BEFORE: &[u8] = b"invalid_storage_before";
const INVALID_VALUE_AFTER: &[u8] = b"invalid_value_after";

// account and amount to unreserve
const ACCOUNT_WITH_RESERVES: [u8; 20] = hex2array!("3fdb68f3a7f614f0f955343b63e2388fa19cd924");
const RESERVES_AMOUNT: u128 = 10_010_850_000_000_000_000_000;

pub struct PalletReferendaRestoreDeposits;
impl Migration for PalletReferendaRestoreDeposits {
	fn friendly_name(&self) -> &str {
		"MM_PalletReferendaRestoreDeposits"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		// compute the hash of the expected status
		let mut expected_hash: Vec<u8> = Vec::new();
		let mut actual_hash: Vec<u8> = Vec::new();
		// track reads and writes
		let mut num_reads = 0;
		let mut num_writes = 0;
		// track the data that will be used to update the storage
		let mut migration_data: Vec<(i32, Vec<u8>, Vec<u8>)> = Vec::new();

		// compute the complete hash of the expected state (the corrupted one)
		// compute the complete hash of the current state
		// if they are the same, it means tha the migration must be executed
		// NOTE: the referenda in the corrupted state are long closed so they cannot be updated anymore,
		// if the migration runs successfully once, the state will never match the corrupted state,
		// this ensures idempotence of the changes
		for (ref_index, ref_storage_key, storage_value_before, storage_value_after) in
			CORRUPTED_REFERENDA
		{
			let key = from_hex(ref_storage_key).unwrap_or_else(|_| {
				log::error!("cannot deserialize from_hex(ref_storage_key)");
				INVALID_STORAGE_KEY.to_vec()
			});

			let value_before = from_hex(storage_value_before).unwrap_or_else(|_| {
				log::error!("cannot deserialize from_hex(storage_value_before)");
				INVALID_VALUE_BEFORE.to_vec()
			});

			// validate also the data that will be written later
			let value_after = from_hex(storage_value_after).unwrap_or(INVALID_VALUE_AFTER.to_vec());
			if value_after == INVALID_VALUE_AFTER {
				log::error!("cannot deserialize from_hex(storage_value_after)");
				return <Runtime as frame_system::Config>::DbWeight::get().reads(num_reads);
			}

			// record the migration data
			migration_data.push((*ref_index, key.clone(), value_after));

			let data = [expected_hash, key.clone(), value_before].concat();
			expected_hash = frame_support::Blake2_256::hash(&data).to_vec();

			let value_current = get_raw(&key);
			num_reads += 1;
			if value_current.is_none() {
				log::error!("storage for referenda {:?} not found", ref_index);
				return <Runtime as frame_system::Config>::DbWeight::get().reads(num_reads);
			}

			let data = [actual_hash, key, value_current.unwrap()].concat();
			actual_hash = frame_support::Blake2_256::hash(&data).to_vec();
		}

		if expected_hash != actual_hash {
			// the migration was already executed, track the reads
			log::warn!(
				"expected partial state hash {:?} doesn't match actual {:x?}, \
				has the migration been already applied?",
				sp_core::hexdisplay::HexDisplay::from(&expected_hash),
				sp_core::hexdisplay::HexDisplay::from(&actual_hash),
			);
			return <Runtime as frame_system::Config>::DbWeight::get().reads(num_reads);
		}

		log::info!(
			"matched expected partial state hash {:?} with actual {:x?}",
			sp_core::hexdisplay::HexDisplay::from(&expected_hash),
			sp_core::hexdisplay::HexDisplay::from(&actual_hash),
		);

		// update the storage
		for (ref_index, ref_storage_key, storage_value_after) in migration_data {
			log::info!("restoring deposits for referenda {}", ref_index);
			put_raw(&ref_storage_key, &storage_value_after);
			num_writes += 1;
		}


		// unlock poopmaster reserves
		// let mut account_with_reserves_bytes: &mut [u8] = &[..];
		let account_with_reserves = <Runtime as frame_system::Config>::AccountId::from(ACCOUNT_WITH_RESERVES);
		frame_system::Account::<Runtime>::mutate(account_with_reserves, |account| {
			log::info!("mutate: Current reserve: {}", account.data.reserved);
			account.data.reserved = account.data.reserved.saturating_sub(RESERVES_AMOUNT);
			log::info!("mutate: New reserve: {}", account.data.reserved);
		});

		// track the read/write
		num_reads += 1;
		num_writes += 1;

		<Runtime as frame_system::Config>::DbWeight::get().reads_writes(num_reads, num_writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		// check that the state is the one expected before runtime
		for (ref_number, ref_storage_key, wrong_storage_value, _) in CORRUPTED_REFERENDA {
			let key = from_hex(ref_storage_key).map_err(|_| {
				sp_runtime::DispatchError::Other(
					"pre_upgrade: cannot decode from_hex(ref_storage_key)",
				)
			})?;
			let val_expected = from_hex(wrong_storage_value).map_err(|_| {
				sp_runtime::DispatchError::Other(
					"pre_upgrade: cannot decode from_hex(wrong_storage_value)",
				)
			})?;
			let val_got = get_raw(&key).ok_or_else(|| {
				log::error!("for referenda {}", ref_number);
				sp_runtime::DispatchError::Other("pre_upgrade: cannot decode get_raw(&key)")
			})?;

			if val_expected != val_got {
				return Err(sp_runtime::DispatchError::Other(
					"pre_upgrade: unexpected storage value",
				));
			}
		}
		let account_with_reserves = <Runtime as frame_system::Config>::AccountId::from(ACCOUNT_WITH_RESERVES);
		let account_info_before = frame_system::Account::<Runtime>::get(account_with_reserves);

		// Returning the account info before changing the reserve value
		Ok(account_info_before.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state_before: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		// check that the state is the one expected before runtime
		for (ref_num, ref_storage_key, _, correct_storage_value) in CORRUPTED_REFERENDA {
			let key = from_hex(ref_storage_key).map_err(|_| {
				sp_runtime::DispatchError::Other(
					"post_upgrade: cannot decode from_hex(ref_storage_key)",
				)
			})?;
			let val_expected = from_hex(correct_storage_value).map_err(|_| {
				sp_runtime::DispatchError::Other(
					"post_upgrade: cannot decode from_hex(correct_storage_value)",
				)
			})?;
			let val_got = get_raw(&key).ok_or_else(|| {
				sp_runtime::DispatchError::Other("post_upgrade: cannot read get_raw(&key)")
			})?;

			if val_expected != val_got {
				return Err(sp_runtime::DispatchError::Other(
					"post_upgrade: unexpected storage value",
				));
			}
			log::info!("referenda {} updated correctly", ref_num);
		}
		let account_with_reserves = <Runtime as frame_system::Config>::AccountId::from(ACCOUNT_WITH_RESERVES);
		let reserve_after = frame_system::Account::<Runtime>::get(account_with_reserves)
			.data
			.reserved;

		let mut state_before = state_before;
		let reserve_before = AccountInfo::<Index, pallet_balances::AccountData<Balance>>::decode(&mut state_before.as_slice())
			.expect("Cannot decode AccountInfo from state_before")
			.data
			.reserved;

		let reserve_differance = reserve_before - reserve_after;

		log::info!("Account with reserve: {:?}", account_with_reserves);
		log::info!("reserve before {} reserve after {}", reserve_before, reserve_after);
		ensure!(reserve_differance == RESERVES_AMOUNT, "The reserve_differance doesn't match RESERVES_AMOUNT");

		Ok(())
	}
}
