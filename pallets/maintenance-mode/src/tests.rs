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
use crate::mock::{
	events, mock_events, ExtBuilder, MaintenanceMode, RuntimeCall as OuterCall, RuntimeOrigin, Test,
};
use crate::{Call, Error, Event, ExecutiveHooks};
use cumulus_primitives_core::DmpMessageHandler;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Dispatchable,
	traits::{OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade},
	weights::Weight,
};

#[test]
fn can_remark_during_normal_operation() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = frame_system::Call::remark { remark: vec![] }.into();
		assert_ok!(call.dispatch(RuntimeOrigin::signed(1)));
	})
}

#[test]
fn cannot_remark_during_maintenance_mode() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = frame_system::Call::remark { remark: vec![] }.into();
			assert_noop!(
				call.dispatch(RuntimeOrigin::signed(1)),
				frame_system::Error::<Test>::CallFiltered
			);
		})
}

#[test]
fn can_enter_maintenance_mode() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = Call::enter_maintenance_mode {}.into();
		assert_ok!(call.dispatch(RuntimeOrigin::root()));

		assert_eq!(events(), vec![Event::EnteredMaintenanceMode,]);
	})
}

#[test]
fn cannot_enter_maintenance_mode_from_wrong_origin() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::enter_maintenance_mode {}.into();
			assert_noop!(
				call.dispatch(RuntimeOrigin::signed(1)),
				frame_system::Error::<Test>::CallFiltered
			);
		})
}

#[test]
fn cannot_enter_maintenance_mode_when_already_in_it() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::enter_maintenance_mode {}.into();
			assert_noop!(
				call.dispatch(RuntimeOrigin::root()),
				Error::<Test>::AlreadyInMaintenanceMode
			);
		})
}

#[test]
fn can_resume_normal_operation() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::resume_normal_operation {}.into();
			assert_ok!(call.dispatch(RuntimeOrigin::root()));

			assert_eq!(events(), vec![Event::NormalOperationResumed,]);
		})
}

#[test]
fn cannot_resume_normal_operation_from_wrong_origin() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::resume_normal_operation {}.into();
			assert_noop!(
				call.dispatch(RuntimeOrigin::signed(1)),
				frame_system::Error::<Test>::CallFiltered
			);
		})
}

#[test]
fn cannot_resume_normal_operation_while_already_operating_normally() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = Call::resume_normal_operation {}.into();
		assert_noop!(
			call.dispatch(RuntimeOrigin::root()),
			Error::<Test>::NotInMaintenanceMode
		);
	})
}

#[cfg(feature = "xcm-support")]
#[test]
fn normal_dmp_in_non_maintenance() {
	ExtBuilder::default()
		.with_maintenance_mode(false)
		.build()
		.execute_with(|| {
			assert_eq!(
				MaintenanceMode::handle_dmp_messages(vec![].into_iter(), Weight::from_ref_time(1)),
				Weight::zero()
			);
		})
}

#[cfg(feature = "xcm-support")]
#[test]
fn maintenance_dmp_in_maintenance() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			assert_eq!(
				MaintenanceMode::handle_dmp_messages(vec![].into_iter(), Weight::from_ref_time(1)),
				Weight::from_ref_time(1)
			);
		})
}

#[test]
fn normal_hooks_in_non_maintenance() {
	ExtBuilder::default()
		.with_maintenance_mode(false)
		.build()
		.execute_with(|| {
			assert_eq!(
				ExecutiveHooks::<Test>::on_idle(0, Weight::zero()),
				Weight::zero()
			);
			assert_eq!(ExecutiveHooks::<Test>::on_initialize(0), Weight::zero());
			assert_eq!(ExecutiveHooks::<Test>::on_runtime_upgrade(), Weight::zero());
			ExecutiveHooks::<Test>::on_finalize(0);
			ExecutiveHooks::<Test>::offchain_worker(0);

			assert_eq!(
				mock_events(),
				[
					crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnIdle,
					crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnInitialize,
					crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnRuntimeUpgrade,
					crate::mock::mock_pallet_maintenance_hooks::Event::NormalOnFinalize,
					crate::mock::mock_pallet_maintenance_hooks::Event::NormalOffchainWorker
				]
			);
		})
}

#[test]
fn maintenance_hooks_in_maintenance() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			assert_eq!(
				ExecutiveHooks::<Test>::on_idle(0, Weight::zero()),
				Weight::from_ref_time(1)
			);
			assert_eq!(
				ExecutiveHooks::<Test>::on_initialize(0),
				Weight::from_ref_time(1)
			);
			assert_eq!(
				ExecutiveHooks::<Test>::on_runtime_upgrade(),
				Weight::from_ref_time(1)
			);

			ExecutiveHooks::<Test>::on_finalize(0);
			ExecutiveHooks::<Test>::offchain_worker(0);
			assert_eq!(
				mock_events(),
				[
					crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnIdle,
					crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnInitialize,
					crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnRuntimeUpgrade,
					crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOnFinalize,
					crate::mock::mock_pallet_maintenance_hooks::Event::MaintenanceOffchainWorker
				]
			);
		})
}
