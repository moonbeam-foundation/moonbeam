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
use crate::mock::{Call as OuterCall, ExtBuilder, Origin, Test};
use frame_support::{assert_noop, assert_ok, dispatch::Dispatchable, storage::IterableStorageMap};
use sp_runtime::DispatchError;

#[test]
fn empty_genesis_works() {
	ExtBuilder::default()
		.build()
		.execute_with(|| assert_eq!(pallet_proxy::Proxies::<Test>::iter().count(), 0))
}

#[test]
fn non_empty_genesis_works() {
	ExtBuilder::default().build().execute_with(|| todo!())
}

#[test]
fn genesis_proxies_reserve_deposits() {
	ExtBuilder::default().build().execute_with(|| todo!())
}
