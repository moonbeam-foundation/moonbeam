// Copyright 2019-2022 PureStake Inc.
// This file is 	part of Moonbeam.

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

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use evm::ExitReason;
use fp_evm::{
	Context, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, Transfer,
};
use frame_support::{
	ensure,
	storage::types::{StorageMap, ValueQuery},
	traits::{Get, StorageInstance},
	Blake2_128Concat,
};
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, H256, U256};
use sp_io::hashing::keccak_256;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Storage prefix for nonces.
pub struct Nonces;

impl StorageInstance for Nonces {
	const STORAGE_PREFIX: &'static str = "Nonces";

	fn pallet_prefix() -> &'static str {
		"PrecompileCallPermit"
	}
}

/// Storage type used to store EIP2612 nonces.
pub type NoncesStorage = StorageMap<
	Nonces,
	// From
	Blake2_128Concat,
	H160,
	// Nonce
	U256,
	ValueQuery,
>;

/// EIP712 permit typehash.
pub const PERMIT_TYPEHASH: [u8; 32] = keccak256!(
	"CallPermit(address from,address to,uint256 value,bytes data,uint64 gaslimit\
,uint256 nonce,uint256 deadline)"
);

/// EIP712 permit domain used to compute an individualized domain separator.
const PERMIT_DOMAIN: [u8; 32] = keccak256!(
	"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Dispatch = "dispatch(address,address,uint256,bytes,uint64,uint256,uint8,bytes32,bytes32)",
	Nonces = "nonces(address)",
	DomainSeparator = "DOMAIN_SEPARATOR()",
}

/// Precompile allowing to issue and dispatch call permits for gasless transactions.
/// A user can sign a permit for a call that can be dispatched and paid by another user or
/// smart contract.
pub struct CallPermitPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for CallPermitPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + pallet_timestamp::Config,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::Dispatch => FunctionModifier::NonPayable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			Action::Dispatch => Self::dispatch(handle),
			Action::Nonces => Self::nonces(handle),
			Action::DomainSeparator => Self::domain_separator(handle),
		}
	}
}

impl<Runtime> CallPermitPrecompile<Runtime>
where
	Runtime: pallet_evm::Config + pallet_timestamp::Config,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	fn compute_domain_separator(address: H160) -> [u8; 32] {
		let name: H256 = keccak_256(b"Call Permit Precompile").into();
		let version: H256 = keccak256!("1").into();
		let chain_id: U256 = Runtime::ChainId::get().into();

		let domain_separator_inner = EvmDataWriter::new()
			.write(H256::from(PERMIT_DOMAIN))
			.write(name)
			.write(version)
			.write(chain_id)
			.write(Address(address))
			.build();

		keccak_256(&domain_separator_inner).into()
	}

	pub fn generate_permit(
		address: H160,
		from: H160,
		to: H160,
		value: U256,
		data: Vec<u8>,
		gaslimit: u64,
		nonce: U256,
		deadline: U256,
	) -> [u8; 32] {
		let domain_separator = Self::compute_domain_separator(address);

		let permit_content = EvmDataWriter::new()
			.write(H256::from(PERMIT_TYPEHASH))
			.write(Address(from))
			.write(Address(to))
			.write(value)
			// bytes are encoded as the keccak_256 of the content
			.write(H256::from(keccak_256(&data)))
			.write(gaslimit)
			.write(nonce)
			.write(deadline)
			.build();
		let permit_content = keccak_256(&permit_content);

		let mut pre_digest = Vec::with_capacity(2 + 32 + 32);
		pre_digest.extend_from_slice(b"\x19\x01");
		pre_digest.extend_from_slice(&domain_separator);
		pre_digest.extend_from_slice(&permit_content);
		keccak_256(&pre_digest)
	}

	pub fn dispatch_inherent_cost() -> u64 {
		3_000 // cost of ECRecover precompile for reference
			+ RuntimeHelper::<Runtime>::db_read_gas_cost() * 2 // we read nonce and timestamp
			+ RuntimeHelper::<Runtime>::db_write_gas_cost() // we write nonce
	}

	fn dispatch(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(Self::dispatch_inherent_cost())?;

		// PARSE INPUT
		let mut input = handle.read_input()?;
		let from = input.read::<Address>()?.0;
		let to = input.read::<Address>()?.0;
		let value: U256 = input.read()?;
		let data = input.read::<Bytes>()?.0;
		let gas_limit: u64 = input.read()?;
		let deadline: U256 = input.read()?;
		let v: u8 = input.read()?;
		let r: H256 = input.read()?;
		let s: H256 = input.read()?;

		// ENSURE GASLIMIT IS SUFFICIENT
		let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

		let total_cost = gas_limit
			.checked_add(call_cost)
			.ok_or_else(|| revert("call require too much gas (u64 overflow)"))?;

		if total_cost > handle.remaining_gas() {
			return Err(revert("gaslimit is too low to dispatch provided call"));
		}

		// VERIFY PERMIT

		// pallet_timestamp is in ms while Ethereum use second timestamps.
		let timestamp: U256 = (pallet_timestamp::Pallet::<Runtime>::get()).into() / 1000;
		ensure!(deadline >= timestamp, revert("permit expired"));

		let nonce = NoncesStorage::get(from);

		let permit = Self::generate_permit(
			handle.context().address,
			from,
			to,
			value,
			data.clone(),
			gas_limit,
			nonce,
			deadline,
		);

		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(&r.as_bytes());
		sig[32..64].copy_from_slice(&s.as_bytes());
		sig[64] = v;

		let signer = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &permit)
			.map_err(|_| revert("invalid permit"))?;
		let signer = H160::from(H256::from_slice(keccak_256(&signer).as_slice()));

		ensure!(
			signer != H160::zero() && signer == from,
			revert("invalid permit")
		);

		NoncesStorage::insert(from, nonce + U256::one());

		// DISPATCH CALL
		let sub_context = Context {
			caller: from,
			address: to.clone(),
			apparent_value: value,
		};

		let transfer = if value.is_zero() {
			None
		} else {
			Some(Transfer {
				source: from,
				target: to.clone(),
				value,
			})
		};

		let (reason, output) =
			handle.call(to, transfer, data, Some(gas_limit), false, &sub_context);

		match reason {
			ExitReason::Error(exit_status) => Err(PrecompileFailure::Error { exit_status }),
			ExitReason::Fatal(exit_status) => Err(PrecompileFailure::Fatal { exit_status }),
			ExitReason::Revert(_) => Err(revert(output)),
			ExitReason::Succeed(_) => Ok(succeed(output)),
		}
	}

	fn nonces(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = handle.read_input()?;
		let from: H160 = input.read::<Address>()?.into();

		let nonce = NoncesStorage::get(from);

		Ok(succeed(EvmDataWriter::new().write(nonce).build()))
	}

	fn domain_separator(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let domain_separator: H256 =
			Self::compute_domain_separator(handle.context().address).into();

		Ok(succeed(
			EvmDataWriter::new().write(domain_separator).build(),
		))
	}
}
