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
use crate::mock::{events, Call, ExtBuilder, Migrations, Origin, System};
use crate::Event;
use frame_support::{assert_ok, dispatch::Dispatchable, traits::OnRuntimeUpgrade};

#[test]
fn genesis_builder_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(System::events().is_empty());
	})
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

//TODO test for multi-block migration
// we would need to make one take more than a block worth of weight for that.

#[test]
fn can_remark_when_not_migrating() {
	ExtBuilder::default().build().execute_with(|| {
		let call: Call = frame_system::Call::remark(vec![]).into();
		assert_ok!(call.dispatch(Origin::signed(1)));
	})
}

// TODO Once we have multiblock migration testing, test that calls are disabled during migrations
// the flow would be: start a migration, try to dispatch, ensure the dispatch failed.
