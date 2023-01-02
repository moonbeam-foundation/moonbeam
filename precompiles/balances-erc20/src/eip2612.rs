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

use super::*;
use frame_support::{ensure, traits::Get};
use sp_core::H256;
use sp_io::hashing::keccak_256;
use sp_std::vec::Vec;

/// EIP2612 permit typehash.
pub const PERMIT_TYPEHASH: [u8; 32] = keccak256!(
	"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
);

/// EIP2612 permit domain used to compute an individualized domain separator.
const PERMIT_DOMAIN: [u8; 32] = keccak256!(
	"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);

pub struct Eip2612<Runtime, Metadata, Instance = ()>(PhantomData<(Runtime, Metadata, Instance)>);

impl<Runtime, Metadata, Instance> Eip2612<Runtime, Metadata, Instance>
where
	Metadata: Erc20Metadata,
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	pub fn compute_domain_separator(address: H160) -> [u8; 32] {
		let name: H256 = keccak_256(Metadata::name().as_bytes()).into();

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
		owner: H160,
		spender: H160,
		value: U256,
		nonce: U256,
		deadline: U256,
	) -> [u8; 32] {
		let domain_separator = Self::compute_domain_separator(address);

		let permit_content = EvmDataWriter::new()
			.write(H256::from(PERMIT_TYPEHASH))
			.write(Address(owner))
			.write(Address(spender))
			.write(value)
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

	// Translated from
	// https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2ERC20.sol#L81
	pub(crate) fn permit(
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
		value: U256,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let owner: H160 = owner.into();
		let spender: H160 = spender.into();

		// pallet_timestamp is in ms while Ethereum use second timestamps.
		let timestamp: U256 = (pallet_timestamp::Pallet::<Runtime>::get()).into() / 1000;

		ensure!(deadline >= timestamp, revert("Permit expired"));

		let nonce = NoncesStorage::<Instance>::get(owner);

		let permit = Self::generate_permit(
			handle.context().address,
			owner,
			spender,
			value,
			nonce,
			deadline,
		);

		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(&r.as_bytes());
		sig[32..64].copy_from_slice(&s.as_bytes());
		sig[64] = v;

		let signer = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &permit)
			.map_err(|_| revert("Invalid permit"))?;
		let signer = H160::from(H256::from_slice(keccak_256(&signer).as_slice()));

		ensure!(
			signer != H160::zero() && signer == owner,
			revert("Invalid permit")
		);

		NoncesStorage::<Instance>::insert(owner, nonce + U256::one());

		{
			let amount =
				Erc20BalancesPrecompile::<Runtime, Metadata, Instance>::u256_to_amount(value)
					.unwrap_or_else(|_| Bounded::max_value());

			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			ApprovesStorage::<Runtime, Instance>::insert(owner, spender, amount);
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_APPROVAL,
			owner,
			spender,
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		Ok(())
	}

	pub(crate) fn nonces(handle: &mut impl PrecompileHandle, owner: Address) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let owner: H160 = owner.into();

		Ok(NoncesStorage::<Instance>::get(owner))
	}

	pub(crate) fn domain_separator(handle: &mut impl PrecompileHandle) -> EvmResult<H256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(Self::compute_domain_separator(handle.context().address).into())
	}
}
