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
	alice, bob, charlie, deploy_addresses, genesis, last_event, root_address, Event as TestEvent,
	Origin, Test, TokenFactory,
};
use crate::{pallet::TokenMinter, Error, Event};
use frame_support::{assert_err, assert_noop, assert_ok};

#[test]
fn registration() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		let tokens_to_register = 10u8;
		let addresses = deploy_addresses();
		for i in 0..tokens_to_register {
			let address = addresses[i as usize];
			assert!(!TokenFactory::exists(&i));
			assert_ok!(TokenFactory::register_token(Origin::root(), i));
			assert_eq!(
				last_event(),
				TestEvent::token_factory(Event::Registered(i, address))
			);
			assert!(TokenFactory::exists(&i));
			assert_eq!(TokenFactory::contract_address(i).unwrap(), address);
		}
	});
}

#[test]
fn minting() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_noop!(
			TokenFactory::mint(0u8, alice(), 10000),
			Error::<Test>::IdNotClaimed
		);
		assert_ok!(TokenFactory::mint(1u8, alice(), 10000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Minted(1, alice(), 10000))
		);
		assert_ok!(TokenFactory::mint(1u8, bob(), 10000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Minted(1, bob(), 10000))
		);
		assert_ok!(TokenFactory::mint(1u8, charlie(), 10000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Minted(1, charlie(), 10000))
		);
		assert_eq!(TokenFactory::total_issuance(1u8).unwrap(), 30000);
		assert_eq!(TokenFactory::balance_of(1u8, alice()).unwrap(), 10000);
		assert_eq!(TokenFactory::balance_of(1u8, bob()).unwrap(), 10000);
		assert_eq!(TokenFactory::balance_of(1u8, charlie()).unwrap(), 10000);
	});
}

#[test]
fn burning() {
	genesis(vec![(root_address(), 5000000000000)]).execute_with(|| {
		assert_ok!(TokenFactory::register_token(Origin::root(), 1u8));
		assert_noop!(
			TokenFactory::burn(0u8, alice(), 10000),
			Error::<Test>::IdNotClaimed
		);
		// not a noop because we still iterate the nonce when we get balance_of before burning
		assert_err!(
			TokenFactory::burn(1u8, alice(), 5000),
			Error::<Test>::NotEnoughBalanceToBurn
		);
		assert_ok!(TokenFactory::mint(1u8, alice(), 10000));
		assert_err!(
			TokenFactory::burn(1u8, bob(), 5000),
			Error::<Test>::NotEnoughBalanceToBurn
		);
		assert_ok!(TokenFactory::mint(1u8, bob(), 10000));
		assert_err!(
			TokenFactory::burn(1u8, charlie(), 5000),
			Error::<Test>::NotEnoughBalanceToBurn
		);
		assert_ok!(TokenFactory::mint(1u8, charlie(), 10000));
		assert_ok!(TokenFactory::burn(1u8, alice(), 5000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Burned(1, alice(), 5000))
		);
		assert_ok!(TokenFactory::burn(1u8, bob(), 5000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Burned(1, bob(), 5000))
		);
		assert_ok!(TokenFactory::burn(1u8, charlie(), 10000));
		assert_eq!(
			last_event(),
			TestEvent::token_factory(Event::Burned(1, charlie(), 10000))
		);
		assert_eq!(TokenFactory::total_issuance(1u8).unwrap(), 10000);
		assert_eq!(TokenFactory::balance_of(1u8, alice()).unwrap(), 5000);
		assert_eq!(TokenFactory::balance_of(1u8, bob()).unwrap(), 5000);
		assert_eq!(TokenFactory::balance_of(1u8, charlie()).unwrap(), 0);
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
			TestEvent::token_factory(Event::Registered(1u8, deploy_addresses()[0]))
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
			TestEvent::token_factory(Event::Registered(1u8, deploy_addresses()[0]))
		);
		assert_eq!(TokenFactory::balance_of(1u8, root_address()).unwrap(), 0u64);
		assert_noop!(
			TokenFactory::balance_of(2u8, root_address()),
			Error::<Test>::IdNotClaimed
		);
	});
}
