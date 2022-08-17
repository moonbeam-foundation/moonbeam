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

use crate::{
	mock::{
		Account::{Alice, Bob, Charlie, Precompile},
		Call, Event, ExtBuilder, Origin, PrecompilesValue, ProxyType, Runtime,
	},
	Action,
};
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_proxy::{
	Call as ProxyCall, Event as ProxyEvent, Pallet as ProxyPallet, ProxyDefinition,
};
use precompile_utils::{
	assert_event_emitted, assert_event_not_emitted, prelude::*, solidity, testing::*,
};
use sp_core::H160;
use std::str::from_utf8;

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_selectors_match_with_actions() {
	assert_eq!(Action::AddProxy as u32, 0x74a34dd3);
	assert_eq!(Action::RemoveProxy as u32, 0xfef3f708);
	assert_eq!(Action::RemoveProxies as u32, 0x14a5b5fa);
}

#[test]
fn test_add_proxy_fails_if_invalid_value_for_proxy_type() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::AddProxy)
						.write::<Address>(bob.into())
						.write::<u8>(10u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|o| o == b"proxyType: Failed decoding value to ProxyType");
		})
}

#[test]
fn test_add_proxy_fails_if_duplicate_proxy() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::AddProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|o| o == b"Cannot add more than one proxy");
		})
}

#[test]
fn test_add_proxy_fails_if_less_permissive_proxy() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::AddProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Nothing as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|o| o == b"Cannot add more than one proxy");
		})
}

#[test]
fn test_add_proxy_fails_if_more_permissive_proxy() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::AddProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Any as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|o| o == b"Cannot add more than one proxy");
		})
}

#[test]
fn test_add_proxy_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::AddProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(1)
						.build(),
				)
				.execute_returns(vec![]);
			assert_event_emitted!(Event::Proxy(ProxyEvent::ProxyAdded {
				delegator: Alice,
				delegatee: Bob,
				proxy_type: ProxyType::Something,
				delay: 1,
			}));

			let proxies = <ProxyPallet<Runtime>>::proxies(Alice).0;
			assert_eq!(
				proxies,
				vec![ProxyDefinition {
					delegate: Bob,
					proxy_type: ProxyType::Something,
					delay: 1,
				}],
			)
		})
}

#[test]
fn test_remove_proxy_fails_if_invalid_value_for_proxy_type() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RemoveProxy)
						.write::<Address>(bob.into())
						.write::<u8>(10u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|o| o == b"proxyType: Failed decoding value to ProxyType");
		})
}

#[test]
fn test_remove_proxy_fails_if_proxy_not_exist() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RemoveProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_reverts(|output| from_utf8(&output).unwrap().contains("NotFound"));
		})
}

#[test]
fn test_remove_proxy_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RemoveProxy)
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_returns(vec![]);
			assert_event_emitted!(Event::Proxy(ProxyEvent::ProxyRemoved {
				delegator: Alice,
				delegatee: Bob,
				proxy_type: ProxyType::Something,
				delay: 0,
			}));

			let proxies = <ProxyPallet<Runtime>>::proxies(Alice).0;
			assert_eq!(proxies, vec![])
		})
}

#[test]
fn test_remove_proxies_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Charlie,
				proxy_type: ProxyType::Any,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RemoveProxies).build(),
				)
				.execute_returns(vec![]);

			let proxies = <ProxyPallet<Runtime>>::proxies(Alice).0;
			assert_eq!(proxies, vec![])
		})
}

#[test]
fn test_remove_proxies_succeeds_when_no_proxy_exists() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RemoveProxies).build(),
				)
				.execute_returns(vec![]);

			let proxies = <ProxyPallet<Runtime>>::proxies(Alice).0;
			assert_eq!(proxies, vec![])
		})
}

#[test]
fn test_is_proxy_returns_false_if_not_proxy() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			let bob: H160 = Bob.into();
			let alice: H160 = Alice.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsProxy)
						.write::<Address>(alice.into())
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_returns(EvmDataWriter::new().write(false).build());
		})
}

#[test]
fn test_is_proxy_returns_false_if_proxy_type_incorrect() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			let alice: H160 = Alice.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsProxy)
						.write::<Address>(alice.into())
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Any as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_returns(EvmDataWriter::new().write(false).build());
		})
}

#[test]
fn test_is_proxy_returns_false_if_proxy_delay_incorrect() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 1u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			let alice: H160 = Alice.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsProxy)
						.write::<Address>(alice.into())
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Any as u8)
						.write::<u32>(0)
						.build(),
				)
				.execute_returns(EvmDataWriter::new().write(false).build());
		})
}

#[test]
fn test_is_proxy_returns_true_if_proxy() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000), (Bob, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 1u64,
			})
			.dispatch(Origin::signed(Alice)));

			let bob: H160 = Bob.into();
			let alice: H160 = Alice.into();
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsProxy)
						.write::<Address>(alice.into())
						.write::<Address>(bob.into())
						.write::<u8>(ProxyType::Something as u8)
						.write::<u32>(1)
						.build(),
				)
				.execute_returns(EvmDataWriter::new().write(true).build());
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["Proxy.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if Action::try_from(selector).is_err() {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}

use sp_core::U256;

#[test]
fn test_nested_evm_bypass_proxy_should_allow_elevating_proxy_type() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 100000000), (Bob, 100000000)])
		.build()
		.execute_with(|| {
			// make Bob a ProxyType::Something for Alice
			assert_ok!(Call::Proxy(ProxyCall::add_proxy {
				delegate: Bob,
				proxy_type: ProxyType::Something,
				delay: 0u64,
			})
			.dispatch(Origin::signed(Alice)));

			// construct the call wrapping the add_proxy precompile to escalate to ProxyType::Any
			let bob: H160 = Bob.into();
			let add_proxy_precompile = EvmDataWriter::new_with_selector(Action::AddProxy)
				.write::<Address>(bob.into())
				.write::<u8>(ProxyType::Any as u8)
				.write::<u32>(0)
				.build();

			let evm_call = Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input: add_proxy_precompile,
				value: U256::zero(),
				gas_limit: u64::max_value(),
				max_fee_per_gas: 0.into(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None,
				access_list: Vec::new(),
			});

			// call the evm call in a proxy call
			assert_ok!(<ProxyPallet<Runtime>>::proxy(
				Origin::signed(Bob.into()),
				Alice.into(),
				None,
				Box::new(evm_call)
			));

			// assert Bob was not assigned ProxyType::Any
			assert_event_not_emitted!(Event::Proxy(ProxyEvent::ProxyAdded {
				delegator: Alice,
				delegatee: Bob,
				proxy_type: ProxyType::Any,
				delay: 0,
			}));
		})
}
