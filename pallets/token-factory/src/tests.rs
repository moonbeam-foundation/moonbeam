// Copyright 2019-2020 PureStake Inc.
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

//! Token Factory Unit Tests
use crate::mock::{
	genesis, last_event, root, Event as TestEvent, Origin, Sudo, Test, TokenFactory,
};
use crate::{pallet::TokenMinter, Event};
use frame_support::{assert_noop, assert_ok, traits::Get};
use sp_runtime::DispatchError;

#[test]
fn registration() {
	genesis(vec![(root(), 5000000000000)]).execute_with(|| {
		assert!(!TokenFactory::exists(&1u8));
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Registered(1u8, root()))
		);
		//assert!(TokenFactory::exists(&1u8));
	});
}
