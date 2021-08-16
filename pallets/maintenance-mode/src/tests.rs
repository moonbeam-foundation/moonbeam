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
use crate::mock::{Call, ExtBuilder, MaintenanceMode, Origin, System};
use crate::Event;
use frame_support::{assert_ok, dispatch::Dispatchable};

#[test]
fn can_remark_during_normal_operation() {
	ExtBuilder::default().build().execute_with(|| {
		let call: Call = frame_system::Call::remark(vec![]).into();
		assert_ok!(call.dispatch(Origin::signed(1)));
	})
}

#[test]
fn cannot_remark_during_maintenance_mode() {
	ExtBuilder::default().build().execute_with(|| {
		let call: Call = frame_system::Call::remark(vec![]).into();

		//TODO test for the right error
		todo!()
	})
}

#[test]
fn can_enter_maintenance_mode() {
	todo!()
}

#[test]
fn cannot_enter_maintenance_mode_when_already_in_it() {
	todo!()
}

#[test]
fn can_resume_normal_operation() {
	todo!()
}

#[test]
fn cannot_resume_normal_operation_while_already_operating_normally() {
	todo!()
}
