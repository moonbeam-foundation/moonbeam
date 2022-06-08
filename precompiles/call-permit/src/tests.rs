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

use crate::mock::{
	balance,
	Account::{Alice, Bob, Charlie, David, Precompile, Revert},
	Call, ExtBuilder, Origin, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::Action;
use evm::ExitReason;
use fp_evm::{ExitError, ExitRevert, ExitSucceed};
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use precompile_utils::{call_cost, testing::*, Address, Bytes, EvmDataWriter, LogExt, LogsBuilder};
use sp_core::{H160, H256, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(from: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: from.into(),
		target: Precompile.into(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None, // Use the next nonce
		access_list: Vec::new(),
	}
}

#[test]
fn selectors() {
	assert_eq!(Action::Dispatch as u32, 0xb5ea0966);
	assert_eq!(Action::Nonces as u32, 0x7ecebe00);
	assert_eq!(Action::DomainSeparator as u32, 0x3644e515);
}
