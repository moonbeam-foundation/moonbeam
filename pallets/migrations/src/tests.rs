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
use crate::mock::{
	events, ExtBuilder, Migrations, System, MockMigration
};
use crate::Event;
use std::sync::{Arc, Mutex};
use frame_support::{
	traits::OnRuntimeUpgrade,
	weights::Weight,
};
use sp_runtime::Perbill;

#[test]
fn genesis_builder_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(System::events().is_empty());
	})
}

#[test]
fn mock_migrations_static_hack_works<'test>() {
	let mut name_fn_called: bool = false;
	let mut step_fn_called: bool = false;

	let mut mgr: crate::mock::MockMigrationManager = Default::default();

	// works:
	// let name_fn: &(FnMut() -> &'static str + Send + Sync) = &|| { "hi" };

	// crate::mock::MOCK_MIGRATIONS_LIST.lock().unwrap()
	mgr
		.registerCallback(
			&mut|| {
				name_fn_called = true;
				"hello, world"
			},
			&mut|_, _| -> (Perbill, Weight) {
				step_fn_called = true;
				(Perbill::zero(), 0u64.into())
			}
		);

	ExtBuilder::default().build().execute_with(|| {
		Migrations::on_runtime_upgrade();
	});

	assert_eq!(name_fn_called, true, "mock migration should call friendly_name()");
	assert_eq!(step_fn_called, true, "mock migration should call step()");
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
			Event::RuntimeUpgradeCompleted(),
		];
		assert_eq!(events(), expected);
	});
}
