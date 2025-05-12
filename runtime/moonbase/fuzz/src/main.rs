/*
 * Harness for asset-hub Kusama
 * Modified for Moonbeam
 */

use codec::{Decode, DecodeLimit, Encode};
use frame_support::{
	dispatch::GetDispatchInfo,
	pallet_prelude::Weight,
	traits::{IntegrityTest, TryState, TryStateSelect},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
};
use fuzzed_runtime::*;
use parachains_common::SLOT_DURATION;
use sp_consensus_aura::{Slot, AURA_ENGINE_ID};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
	traits::{Dispatchable, Header},
	Digest, DigestItem, Storage,
};
use std::time::{Duration, Instant};
use nimbus_primitives::NIMBUS_ENGINE_ID;
use sp_core::H160;
use sp_core::U256;
use sp_state_machine::BasicExternalities as Externalities;
use sp_std::vec;

// The initial timestamp at the start of an input run.
pub const INITIAL_TIMESTAMP: u64 = 0;

/// The maximum number of blocks per fuzzer input.
/// If set to 0, then there is no limit at all.
/// Feel free to set this to a low number (e.g. 4) when you begin your fuzzing campaign and then set it back to 32 once you have good coverage.
pub const MAX_BLOCKS_PER_INPUT: usize = 4;

/// The maximum number of extrinsics per block.
/// If set to 0, then there is no limit at all.
/// Feel free to set this to a low number (e.g. 4) when you begin your fuzzing campaign and then set it back to 0 once you have good coverage.
pub const MAX_EXTRINSICS_PER_BLOCK: usize = 4;

/// Max number of seconds a block should run for.
pub const MAX_TIME_FOR_BLOCK: u64 = 6;

/// The max number of blocks we want the fuzzer to run to between extrinsics.
/// Considering 1 block every 6 seconds, 100_800 blocks correspond to 1 week.
pub const MAX_BLOCK_LAPSE: u32 = 100_800;

// Extrinsic delimiter: `********`
const DELIMITER: [u8; 8] = [42; 8];

// The data structure we use for iterating over extrinsics
pub struct Data<'a> {
	data: &'a [u8],
	pointer: usize,
	size: usize,
}

impl<'a> Data<'a> {
	pub fn from_data(data: &'a [u8]) -> Self {
		Data {
			data,
			pointer: 0,
			size: 0,
		}
	}

	pub fn size_limit_reached(&self) -> bool {
		!(MAX_BLOCKS_PER_INPUT == 0 || MAX_EXTRINSICS_PER_BLOCK == 0)
			&& self.size >= MAX_BLOCKS_PER_INPUT * MAX_EXTRINSICS_PER_BLOCK
	}

	pub fn extract_extrinsics<T: Decode>(&mut self) -> Vec<(Option<u32>, usize, T)> {
		let mut block_count = 0;
		let mut extrinsics_in_block = 0;

		self.filter_map(|data| {
			// We have reached the limit of block we want to decode
			if MAX_BLOCKS_PER_INPUT != 0 && block_count >= MAX_BLOCKS_PER_INPUT {
				return None;
			}
			// lapse is u32 (4 bytes), origin is u16 (2 bytes) -> 6 bytes minimum
			let min_data_len = 4 + 2;
			if data.len() <= min_data_len {
				return None;
			}
			let lapse: u32 = u32::from_ne_bytes(data[0..4].try_into().unwrap());
			let origin: usize = u16::from_ne_bytes(data[4..6].try_into().unwrap()) as usize;
			let mut encoded_extrinsic: &[u8] = &data[6..];

			// If the lapse is in the range [1, MAX_BLOCK_LAPSE] it is valid.
			let maybe_lapse = match lapse {
				1..=MAX_BLOCK_LAPSE => Some(lapse),
				_ => None,
			};
			// We have reached the limit of extrinsics for this block
			if maybe_lapse.is_none()
				&& MAX_EXTRINSICS_PER_BLOCK != 0
				&& extrinsics_in_block >= MAX_EXTRINSICS_PER_BLOCK
			{
				return None;
			}

			match DecodeLimit::decode_with_depth_limit(64, &mut encoded_extrinsic) {
				Ok(decoded_extrinsic) => {
					if maybe_lapse.is_some() {
						block_count += 1;
						extrinsics_in_block = 1;
					} else {
						extrinsics_in_block += 1;
					}
					// We have reached the limit of block we want to decode
					if MAX_BLOCKS_PER_INPUT != 0 && block_count >= MAX_BLOCKS_PER_INPUT {
						return None;
					}
					Some((maybe_lapse, origin, decoded_extrinsic))
				}
				Err(_) => None,
			}
		})
		.collect()
	}
}

impl<'a> Iterator for Data<'a> {
	type Item = &'a [u8];

	fn next(&mut self) -> Option<Self::Item> {
		if self.data.len() <= self.pointer || self.size_limit_reached() {
			return None;
		}
		let next_delimiter = self.data[self.pointer..]
			.windows(DELIMITER.len())
			.position(|window| window == DELIMITER);
		let next_pointer = match next_delimiter {
			Some(delimiter) => self.pointer + delimiter,
			None => self.data.len(),
		};
		let res = Some(&self.data[self.pointer..next_pointer]);
		self.pointer = next_pointer + DELIMITER.len();
		self.size += 1;
		res
	}
}

fn main() {
	let mut endowed_accounts: Vec<AccountId> = (0..10).map(|i| [i; 20].into()).collect();

	// add deadbeef account
	endowed_accounts.push(
		[
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0xde, 0xad, 0xbe, 0xef,
		]
		.into(),
	);
	let _revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	let mut accounts: std::collections::BTreeMap<H160, GenesisAccount> = endowed_accounts
		.clone()
		.into_iter()
		.map(|addr: AccountId| {
			(
				addr.into(),
				GenesisAccount {
					nonce: Default::default(),
					balance: U256::from(1208925819614629174706176u128), // unlimited money),
					storage: Default::default(),
					code: Default::default(),
                    //code: _revert_bytecode.clone(),
				},
			)
		})
		.collect();
	let mut accounts_precompile: std::collections::BTreeMap<H160, GenesisAccount> =
		Precompiles::used_addresses()
			.map(|addr| {
				(
					addr.into(),
					GenesisAccount {
						nonce: Default::default(),
						balance: Default::default(),
						storage: Default::default(),
						//code: revert_bytecode.clone(),
                        code: Default::default(),
					},
				)
			})
			.collect();
	accounts.append(&mut accounts_precompile);

	println!("### print out accounts:: {:?}", accounts);

	let mut mappings = (0..10)
		.map(|i| {
			(
				sp_core::sr25519::Public::unchecked_from([i; 32]).into(),
				[i; 20].into(),
			)
		})
		.collect::<Vec<(sp_core::sr25519::Public, AccountId)>>();
	mappings.push((
		sp_core::sr25519::Public::unchecked_from([
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0xde, 0xad, 0xbe, 0xef,
		]),
		[
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0xde, 0xad, 0xbe, 0xef,
		]
		.into(),
	));

	let genesis_storage: Storage = {

		RuntimeGenesisConfig {
			evm: EVMConfig {
				accounts: accounts,
				..Default::default()
			},

			/*
			author_filter: Default::default(),
			crowdloan_rewards: Default::default(),
			ethereum: Default::default(),
			migrations: Default::default(),
			ethereum_chain_id: Default::default(),
			open_tech_committee_collective: Default::default(),
			parachain_info: Default::default(),
			polkadot_xcm: Default::default(),
			proxy_genesis_companion: Default::default(),
			*/
			balances: BalancesConfig {
				// Configure endowed accounts with initial balance of 1 << 60.
				balances: endowed_accounts
					.iter()
					.cloned()
					.map(|k| (k, 1 << 80))
					.collect(),
			},
			maintenance_mode: MaintenanceModeConfig {
				start_in_maintenance_mode: false,
				..Default::default()
			},
			moonbeam_orbiters: MoonbeamOrbitersConfig {
				min_orbiter_deposit: 1,
			},
			parachain_staking: ParachainStakingConfig {
				candidates: endowed_accounts
					.iter()
					.cloned()
					.map(|k| (k, 1 << 80))
					.collect(),
				// delegations: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
				// blocks_per_round: 10,
				// num_selected_candidates: 1,
				..Default::default()
			},
			author_mapping: AuthorMappingConfig {
				mappings: mappings.into_iter().map(|i| (i.0.into(), i.1)).collect(),
			},

			..Default::default()
		}
		.build_storage()
		.unwrap()
	};

	ziggy::fuzz!(|data: &[u8]| {
		let mut iteratable = crate::Data::from_data(data);

		// Max weight for a block.
		let max_weight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND * 2, 0);

		let extrinsics: Vec<(Option<u32>, usize, RuntimeCall)> =
			iteratable.extract_extrinsics::<RuntimeCall>();

		if extrinsics.is_empty() {
			return;
		}

		// `externalities` represents the state of our mock chain.
		let mut externalities = Externalities::new(genesis_storage.clone());

		let mut current_block: u32 = 1;
		let mut current_weight: Weight = Weight::zero();
		// let mut already_seen = 0; // This must be uncommented if you want to print events
		let mut elapsed: Duration = Duration::ZERO;

		let start_block = |block: u32, lapse: u32| {
			#[cfg(not(fuzzing))]
			println!("\ninitializing block {}", block + lapse);

			let next_block = block + lapse;
			let current_timestamp = INITIAL_TIMESTAMP + u64::from(next_block) * SLOT_DURATION;

			let pre_digest = match current_timestamp {
				INITIAL_TIMESTAMP => Default::default(),
				_ => Digest {
					logs: vec![
						DigestItem::PreRuntime(
							NIMBUS_ENGINE_ID,
							sp_core::sr25519::Public::unchecked_from([1u8; 32]).encode(),
						),
						DigestItem::PreRuntime(
							AURA_ENGINE_ID,
							Slot::from(current_timestamp / SLOT_DURATION).encode(),
						),
					],
				},
			};

			let prev_header = match next_block {
				1 => None,
				_ => Some(Executive::finalize_block()),
			};

			let parent_header = &Header::new(
				next_block + 1,
				Default::default(),
				Default::default(),
				prev_header.clone().map(|x| x.hash()).unwrap_or_default(),
				pre_digest,
			);
			pallet_randomness::vrf::using_fake_vrf(|| Executive::initialize_block(parent_header));

			// We apply the timestamp extrinsic for the current block.
			Executive::apply_extrinsic(UncheckedExtrinsic::new_unsigned(RuntimeCall::Timestamp(
				pallet_timestamp::Call::set {
					now: current_timestamp,
				},
			)))
			.unwrap()
			.unwrap();

			let parachain_validation_data = {
				use cumulus_primitives_core::relay_chain::HeadData;
				use cumulus_primitives_core::PersistedValidationData;
				use cumulus_primitives_parachain_inherent::ParachainInherentData;
				use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;

				let parent_head = HeadData(
					prev_header
						.clone()
						.unwrap_or(parent_header.clone())
						.encode(),
				);
				let sproof_builder = RelayStateSproofBuilder {
					para_id: 100.into(),
					current_slot: Slot::from(2 * current_timestamp / SLOT_DURATION),
					included_para_head: Some(parent_head.clone()),
					..Default::default()
				};

				let (relay_parent_storage_root, relay_chain_state) =
					sproof_builder.into_state_root_and_proof();
				let data = ParachainInherentData {
					validation_data: PersistedValidationData {
						parent_head,
						relay_parent_number: next_block,
						relay_parent_storage_root,
						max_pov_size: 1000,
					},
					relay_chain_state,
					downward_messages: Default::default(),
					horizontal_messages: Default::default(),
				};
				cumulus_pallet_parachain_system::Call::set_validation_data { data }
			};

			Executive::apply_extrinsic(UncheckedExtrinsic::new_unsigned(
				RuntimeCall::ParachainSystem(parachain_validation_data),
			))
			.unwrap()
			.unwrap();

		    Executive::apply_extrinsic(UncheckedExtrinsic::new_unsigned(
		        RuntimeCall::Randomness(
		            pallet_randomness::Call::set_babe_randomness_results{
		
		            }
		        )))
		    .unwrap()
            .unwrap();

			Executive::apply_extrinsic(UncheckedExtrinsic::new_unsigned(
				RuntimeCall::AuthorInherent(
					pallet_author_inherent::Call::kick_off_authorship_validation {},
				),
			))
			.unwrap()
			.unwrap()

			// Calls that need to be called before each block starts (init_calls) go here
		};

		externalities.execute_with(|| start_block(current_block, 0));

		for (maybe_lapse, origin, extrinsic) in extrinsics {
			// If the lapse is in the range [0, MAX_BLOCK_LAPSE] we finalize the block and initialize
			// a new one.
			if let Some(lapse) = maybe_lapse {
				// We update our state variables
				current_weight = Weight::zero();
				elapsed = Duration::ZERO;

				// We start the next block
				externalities.execute_with(|| start_block(current_block, lapse));
				current_block += lapse;
			}

			// We get the current time for timing purposes.
			let now = Instant::now();

			let mut call_weight = Weight::zero();
			// We compute the weight to avoid overweight blocks.
			externalities.execute_with(|| {
				call_weight = extrinsic.get_dispatch_info().weight;
			});

			current_weight = current_weight.saturating_add(call_weight);
			if current_weight.ref_time() >= max_weight.ref_time() {
				#[cfg(not(fuzzing))]
				println!("Skipping because of max weight {max_weight}");
				continue;
			}

			externalities.execute_with(|| {
				//for acc in frame_system::Account::<Runtime>::iter() {
				//    println!("bruno: accounts: {acc:?}");
				//}
				let origin_account = endowed_accounts[origin % endowed_accounts.len()].clone();
				#[cfg(not(fuzzing))]
				{
					println!("\n    origin:     {origin_account:?}");
					println!("    call:       {extrinsic:?}");
				}
				let _res = extrinsic
					.clone()
					.dispatch(RuntimeOrigin::signed(origin_account));
				#[cfg(not(fuzzing))]
				println!("    result:     {_res:?}");

				// Uncomment to print events for debugging purposes
				/*
				#[cfg(not(fuzzing))]
				{
					let all_events = fuzzed_runtime::System::events();
					let events: Vec<_> = all_events.clone().into_iter().skip(already_seen).collect();
					already_seen = all_events.len();
					println!("  events:     {:?}\n", events);
				}
				*/
			});

			elapsed += now.elapsed();
		}

		#[cfg(not(fuzzing))]
		println!("\n  time spent: {elapsed:?}");
		assert!(
			elapsed.as_secs() <= MAX_TIME_FOR_BLOCK,
			"block execution took too much time"
		);

		// We end the final block
		externalities.execute_with(|| {
			// Finilization
			Executive::finalize_block();
			// Invariants
			#[cfg(not(fuzzing))]
			println!("\ntesting invariants for block {current_block}");
			<AllPalletsWithSystem as TryState<BlockNumber>>::try_state(
				current_block,
				TryStateSelect::All,
			)
			.unwrap();
		});

		// After execution of all blocks.
		externalities.execute_with(|| {
            // We keep track of the sum of balance of accounts
            let mut counted_free = 0;
            let mut counted_reserved = 0;

            for acc in frame_system::Account::<Runtime>::iter() {
                // Check that the consumer/provider state is valid.
                let acc_consumers = acc.1.consumers;
                let acc_providers = acc.1.providers;
                assert!(!(acc_consumers > 0 && acc_providers == 0), "Invalid state");

                // Increment our balance counts
                counted_free += acc.1.data.free;
                counted_reserved += acc.1.data.reserved;
                // Check that locks and holds are valid.
                let max_lock: Balance = Balances::locks(&acc.0).iter().map(|l| l.amount).max().unwrap_or_default();
                assert_eq!(max_lock, acc.1.data.frozen, "Max lock should be equal to frozen balance");
                let sum_holds: Balance = pallet_balances::Holds::<Runtime>::get(&acc.0).iter().map(|l| l.amount).sum();
                assert!(
                    sum_holds <= acc.1.data.reserved,
                    "Sum of all holds ({sum_holds}) should be less than or equal to reserved balance {}",
                    acc.1.data.reserved
                );
            }

            let total_issuance = pallet_balances::TotalIssuance::<Runtime>::get();
            let counted_issuance = counted_free + counted_reserved;
            // The reason we do not simply use `!=` here is that some balance might be transfered to another chain via XCM.
            // If we find some kind of workaround for this, we could replace `<` by `!=` here and make the check stronger.
            assert!(
                total_issuance <= counted_issuance,
                "Inconsistent total issuance: {total_issuance} but counted {counted_issuance}"
            );

            #[cfg(not(fuzzing))]
            println!("running integrity tests");
            // We run all developer-defined integrity tests
            <AllPalletsWithSystem as IntegrityTest>::integrity_test();
        });
	});
}
