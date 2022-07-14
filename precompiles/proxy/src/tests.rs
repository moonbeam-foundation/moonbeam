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

// use crate::{
// 	mock::{
// 		events,
// 		Account::{self, Alice, Bob, Precompile},
// 		Balances, Call, Proxy, ExtBuilder, Origin, Precompiles, PrecompilesValue, Runtime,
// 	},
// 	Action,
// };
// use pallet_evm::{Call as EvmCall, Event as EvmEvent};
// use precompile_utils::{prelude::*, testing::*};
// use sp_core::{H160, U256};
// use std::{convert::TryInto, str::from_utf8};

// #[test]
// fn test_selector_less_than_four_bytes_reverts() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		precompiles()
// 			.prepare_test(Alice, Precompile, vec![1u8, 2, 3])
// 			.execute_reverts(|output| output == b"tried to parse selector out of bounds");
// 	});
// }

// #[test]
// fn test_unimplemented_selector_reverts() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		precompiles()
// 			.prepare_test(Alice, Precompile, vec![1u8, 2, 3, 4])
// 			.execute_reverts(|output| output == b"unknown selector");
// 	});
// }

// #[test]
// fn test_selectors_match_with_actions() {
// 	assert_eq!(Action::Delegate as u32, 0x0185921e);
// 	assert_eq!(Action::DepositOf as u32, 0xa30305e9);
// 	assert_eq!(Action::FinishedReferendumInfo as u32, 0xb1fd383f);
// 	assert_eq!(Action::LowestUnbaked as u32, 0x0388f282);
// 	assert_eq!(Action::OngoingReferendumInfo as u32, 0x8b93d11a);
// 	assert_eq!(Action::Propose as u32, 0x7824e7d1);
// 	assert_eq!(Action::PublicPropCount as u32, 0x56fdf547);
// 	assert_eq!(Action::RemoveVote as u32, 0x2042f50b);
// 	assert_eq!(Action::Second as u32, 0xc7a76601);
// 	assert_eq!(Action::StandardVote as u32, 0x3f3c21cc);
// 	assert_eq!(Action::UnDelegate as u32, 0xcb37b8ea);
// 	assert_eq!(Action::Unlock as u32, 0x2f6c493c);
// }

// #[test]
// fn propose_works() {
// 	ExtBuilder::default()
// 		.with_balances(vec![(Alice, 1000)])
// 		.build()
// 		.execute_with(|| {
// 			// Construct data to propose empty hash with value 100
// 			let input = EvmDataWriter::new_with_selector(Action::Propose)
// 				.write(sp_core::H256::zero())
// 				.write(100u64)
// 				.build();

// 			// Make sure the call goes through successfully
// 			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

// 			// Assert that the events are as expected
// 			assert_eq!(
// 				events(),
// 				vec![
// 					BalancesEvent::Reserved {
// 						who: Alice,
// 						amount: 100
// 					}
// 					.into(),
// 					DemocracyEvent::Proposed {
// 						proposal_index: 0,
// 						deposit: 100
// 					}
// 					.into(),
// 					EvmEvent::Executed(Precompile.into()).into(),
// 				]
// 			);
// 		})
// }
