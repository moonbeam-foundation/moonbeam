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

//! Unit testing
use {
	crate::{
		democracy_preimages::*,
		mock::{
			events, ExtBuilder, Migrations, MockMigrationManager, Runtime, RuntimeOrigin, System,
		},
		Error, Event,
	},
	frame_support::{assert_err, assert_ok, traits::OnRuntimeUpgrade, weights::Weight, BoundedVec},
	pallet_preimage::RequestStatus,
	sp_runtime::traits::{Get, Hash},
	std::sync::{Arc, Mutex},
};

#[test]
fn genesis_builder_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(System::events().is_empty());
	})
}

// This test ensures that the mock migration mess works, but also serves as a minimal[-ish] example
// of how to use it. See comments within the fn itself for details.
#[test]
fn mock_migrations_static_hack_works() {
	let name_fn_called = Arc::new(Mutex::new(false));
	let step_fn_called = Arc::new(Mutex::new(false));
	let ecb_fn_called = Arc::new(Mutex::new(false));

	// invoke execute_with_mock_migrations(), which will set up the MockMigrationManager properly
	// and provide a valid reference to it in the callbacks we create.
	crate::mock::execute_with_mock_migrations(
		// This callback receives a mutable ref to the mock which we can use to set up the
		// migrations we wish to mock.
		&mut |mgr: &mut MockMigrationManager| {
			let name_fn_called = Arc::clone(&name_fn_called);
			let step_fn_called = Arc::clone(&step_fn_called);

			// For each migration we wish to mock, we should call register_callback(). The
			// callbacks we provide map to pallet-migration's Migration trait functions.
			mgr.register_callback(
				// mock Migration::friendly_name()
				move || {
					*name_fn_called.lock().unwrap() = true;
					"hello, world"
				},
				// mock Migration::step()
				move |_| -> Weight {
					*step_fn_called.lock().unwrap() = true;
					Weight::zero()
				},
			);
		},
		// This callback provides no parameters, but ensures that the MockMigrationManager
		// "singleton" is still valid. Interactions with the pallet should occur here since they
		// will implicitly require MockMigrationManager to be in a valid state.
		&mut || {
			ExtBuilder::with_uncompleted_migrations(vec!["hello, world"])
				.build()
				.execute_with(|| {
					crate::mock::roll_until_upgraded(true);
				});
			*ecb_fn_called.lock().unwrap() = true;
		},
	);

	assert_eq!(
		*name_fn_called.lock().unwrap(),
		true,
		"mock migration should call friendly_name()"
	);
	assert_eq!(
		*step_fn_called.lock().unwrap(),
		true,
		"mock migration should call step()"
	);
	assert_eq!(
		*ecb_fn_called.lock().unwrap(),
		true,
		"mock migration should call ECB callback"
	);
}

#[test]
fn on_runtime_upgrade_returns() {
	ExtBuilder::default().build().execute_with(|| {
		Migrations::on_runtime_upgrade();
	})
}

#[test]
fn on_runtime_upgrade_emits_events() {
	ExtBuilder::default().build().execute_with(|| {
		Migrations::on_runtime_upgrade();

		let expected = vec![
			Event::RuntimeUpgradeStarted(),
			Event::RuntimeUpgradeCompleted {
				weight: Weight::from_ref_time(100000000u64),
			},
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn migration_should_only_be_invoked_once() {
	let num_name_fn_calls = Arc::new(Mutex::new(0usize));
	let num_step_fn_calls = Arc::new(Mutex::new(0usize));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_name_fn_calls = Arc::clone(&num_name_fn_calls);
			let num_step_fn_calls = Arc::clone(&num_step_fn_calls);

			mgr.register_callback(
				move || {
					let mut num_name_fn_calls = num_name_fn_calls.lock().unwrap();
					*num_name_fn_calls += 1;
					"migration1"
				},
				move |_| -> Weight {
					let mut num_step_fn_calls = num_step_fn_calls.lock().unwrap();
					*num_step_fn_calls += 1;
					Weight::from_ref_time(1)
				},
			);
		},
		&mut || {
			ExtBuilder::with_uncompleted_migrations(vec!["migration1"])
				.build()
				.execute_with(|| {
					// roll forward until upgraded, should happen before block even increments
					crate::mock::roll_until_upgraded(true);

					assert_eq!(System::block_number(), 1);
					// name_fn is called once during the genesis build,
					// then once during the runtime upgrade. So that's two times.
					assert_eq!(
						*num_name_fn_calls.lock().unwrap(),
						2,
						"migration name needed twice"
					);
					assert_eq!(
						*num_step_fn_calls.lock().unwrap(),
						1,
						"migration step needed once"
					);
					let mut expected = vec![
						Event::RuntimeUpgradeStarted(),
						Event::MigrationStarted {
							migration_name: "migration1".into(),
						},
						Event::MigrationCompleted {
							migration_name: "migration1".into(),
							consumed_weight: Weight::from_ref_time(1),
						},
						Event::RuntimeUpgradeCompleted {
							weight: Weight::from_ref_time(100000001u64),
						}, // includes reads/writes
					];
					assert_eq!(events(), expected);

					// attempt to roll forward again, block should still not increment, and migration
					// name fn should be called but pallet_migrations should immediately recognize that
					// no work needs to be done (and not call step)
					crate::mock::roll_until_upgraded(true);

					assert_eq!(System::block_number(), 1);
					assert_eq!(
						*num_name_fn_calls.lock().unwrap(),
						3,
						"migration name needed third"
					);
					assert_eq!(
						*num_step_fn_calls.lock().unwrap(),
						1,
						"migration step not needed again"
					);
					expected.append(&mut vec![
						Event::RuntimeUpgradeStarted(),
						Event::RuntimeUpgradeCompleted {
							weight: Weight::from_ref_time(100000000u64),
						},
					]);
					assert_eq!(events(), expected);

					// roll forward a few blocks
					crate::mock::roll_to(3, false);
					assert_eq!(
						*num_name_fn_calls.lock().unwrap(),
						3,
						"migration name not needed again"
					);
					assert_eq!(
						*num_step_fn_calls.lock().unwrap(),
						1,
						"migration step not needed again"
					);
					// assert that no new events have been emitted
					assert_eq!(events(), expected);
				});
		},
	);
}

#[test]
fn on_runtime_upgrade_charges_max_block_weights() {
	ExtBuilder::default().build().execute_with(|| {
		let block_weights: frame_system::limits::BlockWeights =
			<Runtime as frame_system::Config>::BlockWeights::get();
		let weight = Migrations::on_runtime_upgrade();
		assert_eq!(weight, block_weights.max_block);
	})
}

#[test]
fn overweight_migrations_tolerated() {
	// pallet-migrations currently tolerates a migration going over-weight. not only does it
	// tolerate it, but it continues on to the next migration even if it's already overweight.
	//
	// The logic behind this is that we would rather go over-weight and risk a block taking too long
	// (which *might* be "catastrophic") than outright prevent migrations from proceeding (which is
	// certainly "catastrophic").
	//
	// maybe_catastrophic > certainly_catastrophic

	let num_migration1_calls = Arc::new(Mutex::new(0u32));
	let num_migration2_calls = Arc::new(Mutex::new(0u32));
	let num_migration3_calls = Arc::new(Mutex::new(0u32));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_migration1_calls = Arc::clone(&num_migration1_calls);
			let num_migration2_calls = Arc::clone(&num_migration2_calls);
			let num_migration3_calls = Arc::clone(&num_migration3_calls);

			mgr.register_callback(
				move || "migration1",
				move |_| -> Weight {
					*num_migration1_calls.lock().unwrap() += 1;
					// TODO: this is brittle because it assumes it is larger than the value used at
					// the top of process_runtime_upgrades()
					Weight::from_ref_time(1_000_000_000_000u64)
				},
			);

			mgr.register_callback(
				move || "migration2",
				move |_| -> Weight {
					*num_migration2_calls.lock().unwrap() += 1;
					Weight::from_ref_time(1_000_000_000_000u64)
				},
			);

			mgr.register_callback(
				move || "migration3",
				move |_| -> Weight {
					*num_migration3_calls.lock().unwrap() += 1;
					Weight::from_ref_time(1_000_000_000_000u64)
				},
			);
		},
		&mut || {
			ExtBuilder::with_uncompleted_migrations(vec!["migration1", "migration2", "migration3"])
				.build()
				.execute_with(|| {
					Migrations::on_runtime_upgrade();

					assert_eq!(*num_migration1_calls.lock().unwrap(), 1);
					assert_eq!(*num_migration2_calls.lock().unwrap(), 1);
					assert_eq!(*num_migration3_calls.lock().unwrap(), 1);
					assert_eq!(Migrations::is_fully_upgraded(), true);
				});
		},
	);
}

#[cfg(all(test, feature = "try-runtime"))]
fn try_runtime_functions_work() {
	let pre_fn_called = Arc::new(Mutex::new(false));
	let post_fn_called = Arc::new(Mutex::new(false));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let pre_fn_called = Arc::clone(&pre_fn_called);
			let post_fn_called = Arc::clone(&post_fn_called);
			mgr.register_callback_with_try_fns(
				move || "dummy_step",
				move |_| -> Weight { 0u64.into() },
				move || -> Result<(), &'static str> {
					*pre_fn_called.lock().unwrap() = true;
					Ok(())
				},
				move || -> Result<(), &'static str> {
					*post_fn_called.lock().unwrap() = true;
					Ok(())
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				crate::mock::invoke_all_upgrade_hooks();
			});
		},
	);

	assert_eq!(
		*pre_fn_called.lock().unwrap(),
		true,
		"mock migration should call pre_upgrade()"
	);

	assert_eq!(
		*post_fn_called.lock().unwrap(),
		true,
		"mock migration should call post_upgrade()"
	);
}

#[test]
fn preimage_lazy_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup mock preimage
		let data = b"hello world!".to_vec();
		let bounded_data: BoundedVec<_, _> = data.clone().try_into().expect("fits in bound");
		let len = data.len() as u32;
		let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&data);

		DeprecatedDemocracyPreimages::<Runtime>::insert(
			hash,
			PreimageStatus::Available {
				data,
				provider: 42,
				deposit: 142,
				since: 0,
				expiry: None,
			},
		);

		// Call migration
		assert_ok!(Migrations::migrate_democracy_preimage(
			RuntimeOrigin::signed(1),
			hash,
			len,
		));

		// Check migration was successful.
		assert!(DeprecatedDemocracyPreimages::<Runtime>::get(hash).is_none());
		assert_eq!(
			StatusFor::<Runtime>::get(hash),
			Some(RequestStatus::Unrequested {
				deposit: (42, 142),
				len
			})
		);
		assert_eq!(PreimageFor::<Runtime>::get((hash, len)), Some(bounded_data));
	});
}

#[test]
fn preimage_lazy_migration_fails_if_their_is_nothing_to_migrate() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup mock preimage
		let data = b"hello world!".to_vec();
		let len = data.len() as u32;
		let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&data);

		// (we don't insert it, there is nothing to migrate)

		// Call migration
		assert_err!(
			Migrations::migrate_democracy_preimage(RuntimeOrigin::signed(1), hash, len,),
			Error::<Runtime>::PreimageMissing
		);
	});
}

#[test]
fn preimage_lazy_migration_fails_if_preimage_already_exists() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup mock preimage
		let data = b"hello world!".to_vec();
		let len = data.len() as u32;
		let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&data);

		DeprecatedDemocracyPreimages::<Runtime>::insert(
			hash,
			PreimageStatus::Available {
				data,
				provider: 42,
				deposit: 142,
				since: 0,
				expiry: None,
			},
		);

		// Setup mock preimage in new pallet
		StatusFor::<Runtime>::insert(
			hash,
			RequestStatus::Unrequested {
				deposit: (42, 142),
				len,
			},
		);

		// Call migration
		assert_err!(
			Migrations::migrate_democracy_preimage(RuntimeOrigin::signed(1), hash, len,),
			Error::<Runtime>::PreimageAlreadyExists
		);

		// Check there was no migration.
		assert!(DeprecatedDemocracyPreimages::<Runtime>::get(hash).is_some());
	});
}

#[test]
fn preimage_lazy_migration_fails_if_len_hint_is_wrong() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup mock preimage
		let data = b"hello world!".to_vec();
		let len = data.len() as u32;
		let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&data);

		DeprecatedDemocracyPreimages::<Runtime>::insert(
			hash,
			PreimageStatus::Available {
				data,
				provider: 42,
				deposit: 142,
				since: 0,
				expiry: None,
			},
		);

		// Call migration
		assert_err!(
			Migrations::migrate_democracy_preimage(
				RuntimeOrigin::signed(1),
				hash,
				len - 1, // too short !
			),
			Error::<Runtime>::WrongUpperBound
		);

		// Check there was no migration.
		assert!(DeprecatedDemocracyPreimages::<Runtime>::get(hash).is_some());
		assert!(StatusFor::<Runtime>::get(hash).is_none());
		assert!(PreimageFor::<Runtime>::get((hash, len)).is_none());
	});
}

#[test]
fn preimage_lazy_migration_fails_if_preimage_is_too_big() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup mock preimage
		let data = [1; (MAX_SIZE as usize) + 1].to_vec();
		let len = data.len() as u32;
		let hash = <Runtime as frame_system::Config>::Hashing::hash_of(&data);

		DeprecatedDemocracyPreimages::<Runtime>::insert(
			hash,
			PreimageStatus::Available {
				data,
				provider: 42,
				deposit: 142,
				since: 0,
				expiry: None,
			},
		);

		// Call migration
		assert_err!(
			Migrations::migrate_democracy_preimage(RuntimeOrigin::signed(1), hash, len,),
			Error::<Runtime>::PreimageIsTooBig
		);

		// Check there was no migration.
		assert!(DeprecatedDemocracyPreimages::<Runtime>::get(hash).is_some());
		assert!(StatusFor::<Runtime>::get(hash).is_none());
		assert!(PreimageFor::<Runtime>::get((hash, len)).is_none());
	});
}

// TODO: a test to ensure that post_upgrade invokes the same set of migrations that pre_upgrade
// does would be useful
