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
	deploy_address, genesis, last_event, root_address, Event as TestEvent, Origin, Test,
	TokenFactory,
};
use crate::{pallet::TokenMinter, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn registration() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert!(!TokenFactory::exists(&1u8));
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Registered(1u8, deploy_address()))
		);
		assert!(TokenFactory::exists(&1u8));
		assert_eq!(
			TokenFactory::contracts(1u8).unwrap().address,
			deploy_address()
		);
	});
}

#[test]
fn minting() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_ok!(TokenFactory::mint(1u8, root_address(), 10000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Minted(1, root_address(), 10000))
		);
		assert_eq!(TokenFactory::total_issuance(1u8).unwrap(), 10000);
		assert_eq!(
			TokenFactory::balance_of(1u8, root_address()).unwrap(),
			10000
		);
	});
}

#[test]
fn burning() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_ok!(TokenFactory::mint(1u8, root_address(), 10000));
		assert_ok!(TokenFactory::burn(1u8, root_address(), 5000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Burned(1, root_address(), 5000))
		);
		assert_eq!(TokenFactory::total_issuance(1u8).unwrap(), 5000);
		assert_eq!(TokenFactory::balance_of(1u8, root_address()).unwrap(), 5000);
	});
}

#[test]
fn get_total_supply() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_ok!(TokenFactory::total_issuance(1u8));
		// implies that the error event was not emitted in total issuance call
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Registered(1u8, deploy_address()))
		);
		assert_eq!(TokenFactory::total_issuance(1u8).unwrap(), 0u64);
		assert_noop!(
			TokenFactory::total_issuance(2u8),
			Error::<Test>::IdNotClaimed
		);
	});
}

#[test]
fn get_balance_of() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_ok!(TokenFactory::balance_of(1u8, root_address()));
		// implies that the error event was not emitted in total issuance call
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Registered(1u8, deploy_address()))
		);
		assert_eq!(TokenFactory::balance_of(1u8, root_address()).unwrap(), 0u64);
		assert_noop!(
			TokenFactory::balance_of(2u8, root_address()),
			Error::<Test>::IdNotClaimed
		);
	});
}
