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
use frame_support::{
	ensure,
	storage::types::{StorageDoubleMap, ValueQuery},
	traits::{StorageInstance, Time},
	Blake2_128Concat,
};
use pallet_assets::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};
use scale_info::prelude::string::ToString;
use sp_core::H256;
use sp_io::hashing::keccak_256;
use sp_runtime::traits::UniqueSaturatedInto;

/// EIP2612 permit typehash.
pub const PERMIT_TYPEHASH: [u8; 32] = keccak256!(
	"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
);

/// EIP2612 permit domain used to compute an individualized domain separator.
const PERMIT_DOMAIN: [u8; 32] = keccak256!(
	"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
);

/// Associates pallet Instance to a prefix used for the Nonces storage.
/// This trait is implemented for () and the 16 substrate Instance.
pub trait InstanceToPrefix {
	/// Prefix used for the Approves storage.
	type NoncesPrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($instance:ident, $name:literal) => {
		// Using `paste!` we generate a dedicated module to avoid collisions
		// between each instance `Nonces` struct.
		paste::paste! {
			mod [<_impl_prefix_ $instance:snake>] {
				use super::*;

				pub struct Nonces;

				impl StorageInstance for Nonces {
					const STORAGE_PREFIX: &'static str = "Nonces";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}

				impl InstanceToPrefix for $instance {
					type NoncesPrefix = Nonces;
				}
			}
		}
	};
}

// Since the macro expect a `ident` to be used with `paste!` we cannot provide `()` directly.
type Instance0 = ();

impl_prefix!(Instance0, "Erc20Instance0Assets");
impl_prefix!(Instance1, "Erc20Instance1Assets");
impl_prefix!(Instance2, "Erc20Instance2Assets");
impl_prefix!(Instance3, "Erc20Instance3Assets");
impl_prefix!(Instance4, "Erc20Instance4Assets");
impl_prefix!(Instance5, "Erc20Instance5Assets");
impl_prefix!(Instance6, "Erc20Instance6Assets");
impl_prefix!(Instance7, "Erc20Instance7Assets");
impl_prefix!(Instance8, "Erc20Instance8Assets");
impl_prefix!(Instance9, "Erc20Instance9Assets");
impl_prefix!(Instance10, "Erc20Instance10Assets");
impl_prefix!(Instance11, "Erc20Instance11Assets");
impl_prefix!(Instance12, "Erc20Instance12Assets");
impl_prefix!(Instance13, "Erc20Instance13Assets");
impl_prefix!(Instance14, "Erc20Instance14Assets");
impl_prefix!(Instance15, "Erc20Instance15Assets");
impl_prefix!(Instance16, "Erc20Instance16Assets");

/// Storage type used to store EIP2612 nonces.
pub type NoncesStorage<Instance> = StorageDoubleMap<
	<Instance as InstanceToPrefix>::NoncesPrefix,
	// Asset contract address
	Blake2_128Concat,
	H160,
	// Owner
	Blake2_128Concat,
	H160,
	// Nonce
	U256,
	ValueQuery,
>;

pub struct Eip2612<Runtime, Instance: 'static = ()>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Eip2612<Runtime, Instance>
where
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	AssetIdOf<Runtime, Instance>: Display,
	Runtime::AccountId: Into<H160>,
{
	fn compute_domain_separator(address: H160, asset_id: AssetIdOf<Runtime, Instance>) -> [u8; 32] {
		let asset_name = pallet_assets::Pallet::<Runtime, Instance>::name(asset_id.clone());

		let name = if asset_name.is_empty() {
			let mut name = b"Unnamed XC20 #".to_vec();
			name.extend_from_slice(asset_id.to_string().as_bytes());
			name
		} else {
			asset_name
		};

		let name: H256 = keccak_256(&name).into();
		let version: H256 = keccak256!("1").into();
		let chain_id: U256 = Runtime::ChainId::get().into();

		let domain_separator_inner = solidity::encode_arguments((
			H256::from(PERMIT_DOMAIN),
			name,
			version,
			chain_id,
			Address(address),
		));

		keccak_256(&domain_separator_inner)
	}

	pub fn generate_permit(
		address: H160,
		asset_id: AssetIdOf<Runtime, Instance>,
		owner: H160,
		spender: H160,
		value: U256,
		nonce: U256,
		deadline: U256,
	) -> [u8; 32] {
		let domain_separator = Self::compute_domain_separator(address, asset_id);

		let permit_content = solidity::encode_arguments((
			H256::from(PERMIT_TYPEHASH),
			Address(owner),
			Address(spender),
			value,
			nonce,
			deadline,
		));
		let permit_content = keccak_256(&permit_content);

		let mut pre_digest = Vec::with_capacity(2 + 32 + 32);
		pre_digest.extend_from_slice(b"\x19\x01");
		pre_digest.extend_from_slice(&domain_separator);
		pre_digest.extend_from_slice(&permit_content);
		keccak_256(&pre_digest)
	}

	// Translated from
	// https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2ERC20.sol#L81
	#[allow(clippy::too_many_arguments)]
	pub(crate) fn permit(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
		value: U256,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult {
		// NoncesStorage: Blake2_128(16) + contract(20) + Blake2_128(16) + owner(20) + nonce(32)
		handle.record_db_read::<Runtime>(104)?;

		let owner: H160 = owner.into();
		let spender: H160 = spender.into();

		let address = handle.code_address();

		// Blockchain time is in ms while Ethereum use second timestamps.
		let timestamp: u128 =
			<Runtime as pallet_evm::Config>::Timestamp::now().unique_saturated_into();
		let timestamp: U256 = U256::from(timestamp / 1000);

		ensure!(deadline >= timestamp, revert("Permit expired"));

		let nonce = NoncesStorage::<Instance>::get(address, owner);

		let permit = Self::generate_permit(
			address,
			asset_id.clone(),
			owner,
			spender,
			value,
			nonce,
			deadline,
		);

		let mut sig = [0u8; 65];
		sig[0..32].copy_from_slice(r.as_bytes());
		sig[32..64].copy_from_slice(s.as_bytes());
		sig[64] = v;

		let signer = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &permit)
			.map_err(|_| revert("Invalid permit"))?;
		let signer = H160::from(H256::from_slice(keccak_256(&signer).as_slice()));

		ensure!(
			signer != H160::zero() && signer == owner,
			revert("Invalid permit")
		);

		NoncesStorage::<Instance>::insert(address, owner, nonce + U256::one());

		Erc20AssetsPrecompileSet::<Runtime, Instance>::approve_inner(
			asset_id, handle, owner, spender, value,
		)?;

		log3(
			address,
			SELECTOR_LOG_APPROVAL,
			owner,
			spender,
			solidity::encode_event_data(value),
		)
		.record(handle)?;

		Ok(())
	}

	pub(crate) fn nonces(
		_asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<U256> {
		// NoncesStorage: Blake2_128(16) + contract(20) + Blake2_128(16) + owner(20) + nonce(32)
		handle.record_db_read::<Runtime>(104)?;

		let owner: H160 = owner.into();

		let nonce = NoncesStorage::<Instance>::get(handle.code_address(), owner);

		Ok(nonce)
	}

	pub(crate) fn domain_separator(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<H256> {
		// Storage item: AssetMetadata:
		// Blake2_128(16) + AssetId(16) + AssetMetadata[deposit(16) + name(StringLimit)
		// + symbol(StringLimit) + decimals(1) + is_frozen(1)]
		handle.record_db_read::<Runtime>(
			50 + (2 * <Runtime as pallet_assets::Config<Instance>>::StringLimit::get()) as usize,
		)?;

		let domain_separator: H256 =
			Self::compute_domain_separator(handle.code_address(), asset_id).into();

		Ok(domain_separator)
	}
}
