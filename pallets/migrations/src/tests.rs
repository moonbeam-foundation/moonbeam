// Copyright 2019-2021 PureStake Inc.
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
use crate::mock::{events, ExtBuilder, Migrations, MockMigrationManager, System};
use crate::Event;
use frame_support::{
	traits::{OnInitialize, OnRuntimeUpgrade},
	weights::{constants::RocksDbWeight, Weight},
};
use sp_runtime::Perbill;
use std::sync::{Arc, Mutex};

#[test]
fn genesis_builder_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(System::events().is_empty());
	})
}

#[test]
fn mock_migrations_static_hack_works() {
	let name_fn_called = Arc::new(Mutex::new(false));
	let step_fn_called = Arc::new(Mutex::new(false));
	let ecb_fn_called = Arc::new(Mutex::new(false));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let name_fn_called = Arc::clone(&name_fn_called);
			let step_fn_called = Arc::clone(&step_fn_called);

			mgr.register_callback(
				move || {
					*name_fn_called.lock().unwrap() = true;
					"hello, world"
				},
				move |_, _| -> (Perbill, Weight) {
					*step_fn_called.lock().unwrap() = true;
					(Perbill::one(), 0u64.into())
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
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
			Event::RuntimeUpgradeStepped(0u64.into()),
			Event::RuntimeUpgradeCompleted(),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn step_called_until_done() {
	let num_step_calls = Arc::new(Mutex::new(0usize));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_step_calls = Arc::clone(&num_step_calls);

			mgr.register_callback(
				move || "migration1",
				move |_, _| -> (Perbill, Weight) {
					let mut num_step_calls = num_step_calls.lock().unwrap();
					*num_step_calls += 1;
					if *num_step_calls == 10 {
						(Perbill::one(), 0u64.into())
					} else {
						(Perbill::zero(), 0u64.into())
					}
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				crate::mock::roll_until_upgraded(true);
			});
		},
	);

	assert_eq!(
		*num_step_calls.lock().unwrap(),
		10,
		"migration step should be called until done"
	);
}

#[test]
fn migration_progress_should_emit_events() {
	let num_steps = Arc::new(Mutex::new(0usize));

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_steps = Arc::clone(&num_steps);

			mgr.register_callback(
				move || "migration1",
				move |_, _| -> (Perbill, Weight) {
					let mut num_steps = num_steps.lock().unwrap();

					let result: (Perbill, Weight) = match *num_steps {
						0 => (Perbill::from_percent(50), 50),
						1 => (Perbill::from_percent(60), 51),
						2 => (Perbill::from_percent(70), 52),
						3 => (Perbill::from_percent(80), 53),
						4 => (Perbill::from_percent(100), 1),
						_ => {
							unreachable!();
						}
					};

					*num_steps += 1;
					result
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				crate::mock::roll_until_upgraded(true);

				let expected = vec![
					Event::RuntimeUpgradeStarted(),
					Event::MigrationStarted("migration1".into()),
					Event::MigrationStepped("migration1".into(), Perbill::from_percent(50), 50),
					Event::RuntimeUpgradeStepped(50),
					Event::MigrationStepped("migration1".into(), Perbill::from_percent(60), 51),
					Event::RuntimeUpgradeStepped(51),
					Event::MigrationStepped("migration1".into(), Perbill::from_percent(70), 52),
					Event::RuntimeUpgradeStepped(52),
					Event::MigrationStepped("migration1".into(), Perbill::from_percent(80), 53),
					Event::RuntimeUpgradeStepped(53),
					Event::MigrationStepped("migration1".into(), Perbill::from_percent(100), 1),
					Event::MigrationCompleted("migration1".into()),
					Event::RuntimeUpgradeStepped(1),
					Event::RuntimeUpgradeCompleted(),
				];
				assert_eq!(events(), expected);
			});
		},
	);

	assert_eq!(
		*num_steps.lock().unwrap(),
		5,
		"migration step should be called until done"
	);
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
				move |_, _| -> (Perbill, Weight) {
					let mut num_step_fn_calls = num_step_fn_calls.lock().unwrap();
					*num_step_fn_calls += 1;
					(Perbill::one(), 1) // immediately done
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				// roll forward until upgraded, should happen before block even increments
				crate::mock::roll_until_upgraded(true);

				assert_eq!(System::block_number(), 1);
				assert_eq!(
					*num_name_fn_calls.lock().unwrap(),
					1,
					"migration name needed once"
				);
				assert_eq!(
					*num_step_fn_calls.lock().unwrap(),
					1,
					"migration step needed once"
				);
				let mut expected = vec![
					Event::RuntimeUpgradeStarted(),
					Event::MigrationStarted("migration1".into()),
					Event::MigrationStepped("migration1".into(), Perbill::one(), 1),
					Event::MigrationCompleted("migration1".into()),
					Event::RuntimeUpgradeStepped(1),
					Event::RuntimeUpgradeCompleted(),
				];
				assert_eq!(events(), expected);

				// attempt to roll forward again, block should still not increment, and migration
				// name fn should be called but pallet_migrations should immediately recognize that
				// no work needs to be done (and not call step)
				crate::mock::roll_until_upgraded(true);

				assert_eq!(System::block_number(), 1);
				assert_eq!(
					*num_name_fn_calls.lock().unwrap(),
					2,
					"migration name needed twice"
				);
				assert_eq!(
					*num_step_fn_calls.lock().unwrap(),
					1,
					"migration step not needed again"
				);
				expected.append(&mut vec![
					Event::RuntimeUpgradeStarted(),
					Event::RuntimeUpgradeStepped(0),
					Event::RuntimeUpgradeCompleted(),
				]);
				assert_eq!(events(), expected);

				// roll forward a few blocks
				crate::mock::roll_to(3, false);
				assert_eq!(
					*num_name_fn_calls.lock().unwrap(),
					2,
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
fn on_initialize_should_charge_at_least_one_db_read() {
	ExtBuilder::default().build().execute_with(|| {
		// first call to on_runtime_upgrade should flag FullyUpgraded as true
		Migrations::on_runtime_upgrade();
		assert_eq!(Migrations::is_fully_upgraded(), true);

		// and subsequent call to on_initialize should do nothing but read this value and return
		let weight = <Migrations as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(weight, RocksDbWeight::get().reads(1));
	})
}

#[test]
fn on_runtime_upgrade_charges_minimum_two_db_writes() {
	ExtBuilder::default().build().execute_with(|| {
		let mut weight = Migrations::on_runtime_upgrade();

		// substrate seems to add a write to this call, so substract one for our logic
		weight -= RocksDbWeight::get().writes(1);

		assert_eq!(weight, RocksDbWeight::get().writes(2));
	})
}

#[test]
fn only_one_outstanding_test_at_a_time() {
	let num_migration1_calls = Arc::new(Mutex::new(0usize));
	let num_migration2_calls = Arc::new(Mutex::new(0usize));

	// create two migrations. the first will return < Perbill::one() until its 3rd step, which
	// should prevent the second from running. Once it s done, the second should execute.

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_migration1_calls = Arc::clone(&num_migration1_calls);
			let num_migration2_calls = Arc::clone(&num_migration2_calls);

			mgr.register_callback(
				move || "migration1",
				move |_, _| -> (Perbill, Weight) {
					let mut num_migration1_calls = num_migration1_calls.lock().unwrap();
					*num_migration1_calls += 1;

					// this migration is done on its 3rd step
					if *num_migration1_calls < 3 {
						(Perbill::zero(), 0u64.into())
					} else {
						(Perbill::one(), 0u64.into())
					}
				},
			);

			mgr.register_callback(
				move || "migration2",
				move |_, _| -> (Perbill, Weight) {
					let mut num_migration2_calls = num_migration2_calls.lock().unwrap();
					*num_migration2_calls += 1;
					(Perbill::one(), 0u64.into())
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				// first pass should invoke migration1 once and not move on to migration2
				Migrations::on_runtime_upgrade();
				assert_eq!(*num_migration1_calls.lock().unwrap(), 1);
				assert_eq!(*num_migration2_calls.lock().unwrap(), 0);

				// second pass should do the same
				crate::mock::roll_to(2, false);
				assert_eq!(*num_migration1_calls.lock().unwrap(), 2);
				assert_eq!(*num_migration2_calls.lock().unwrap(), 0);

				// third pass should invoke both
				crate::mock::roll_to(3, false);
				assert_eq!(*num_migration1_calls.lock().unwrap(), 3);
				assert_eq!(*num_migration2_calls.lock().unwrap(), 1);

				// and both should be done now
				assert_eq!(Migrations::is_fully_upgraded(), true);
			});
		},
	);
}

#[test]
fn multi_block_migration_flag_works() {
	let num_migration_calls = Arc::new(Mutex::new(0u32));

	// we create a single migration that wants to take more than one block and ensure that it's only
	// called once

	crate::mock::execute_with_mock_migrations(
		&mut |mgr: &mut MockMigrationManager| {
			let num_migration_calls = Arc::clone(&num_migration_calls);

			mgr.is_multi_block = false;

			mgr.register_callback(
				move || "migration1",
				move |_, _| -> (Perbill, Weight) {
					*num_migration_calls.lock().unwrap() += 1;
					(Perbill::zero(), 0u64.into())
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				crate::mock::roll_to(5, true);

				assert_eq!(*num_migration_calls.lock().unwrap(), 1);
				assert_eq!(Migrations::is_fully_upgraded(), false);
			});
		},
	);
}

#[test]
fn overweight_migrations_tolerated() {
	// pallet-migrations currently tolerates a migration going over-weight. not only does it
	// tolerate it, but it continues on to the next migration even if it's already overweight.
	//
	// Now that the pallet can be configured to not support multi-block migrations, this is sort of
	// a feature and not really a bug -- this test case exists to explicitly acknowledge/protect
	// that.
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

			mgr.is_multi_block = false;

			mgr.register_callback(
				move || "migration1",
				move |_, _| -> (Perbill, Weight) {
					*num_migration1_calls.lock().unwrap() += 1;
					// TODO: this is brittle because it assumes it is larger than the value used at
					// the top of process_runtime_upgrades()
					(Perbill::one(), 1_000_000_000_000u64.into())
				},
			);

			mgr.register_callback(
				move || "migration2",
				move |_, _| -> (Perbill, Weight) {
					*num_migration2_calls.lock().unwrap() += 1;
					(Perbill::one(), 1_000_000_000_000u64.into())
				},
			);

			mgr.register_callback(
				move || "migration3",
				move |_, _| -> (Perbill, Weight) {
					*num_migration3_calls.lock().unwrap() += 1;
					(Perbill::one(), 1_000_000_000_000u64.into())
				},
			);
		},
		&mut || {
			ExtBuilder::default().build().execute_with(|| {
				Migrations::on_runtime_upgrade();

				assert_eq!(*num_migration1_calls.lock().unwrap(), 1);
				assert_eq!(*num_migration2_calls.lock().unwrap(), 1);
				assert_eq!(*num_migration3_calls.lock().unwrap(), 1);
				assert_eq!(Migrations::is_fully_upgraded(), true);
			});
		},
	);
}
