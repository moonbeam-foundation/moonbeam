// Copyright 2019-2025 PureStake Inc.
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

use account::AccountId20;
use frame_support::traits::fungible::NativeOrWithId;
use moonbeam_core_primitives::{AssetId, Signature};
use pallet_identity;
use pallet_treasury::ArgumentsFactory;
use sp_runtime::traits::PhantomData;

pub struct BenchmarkHelper<T>(PhantomData<T>);

impl<T> ArgumentsFactory<NativeOrWithId<AssetId>, AccountId20> for BenchmarkHelper<T> {
	fn create_asset_kind(seed: u32) -> NativeOrWithId<AssetId> {
		NativeOrWithId::WithId(seed.into())
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId20 {
		// Avoid generating a zero address
		if seed == [0; 32] {
			return AccountId20::from([1; 32]);
		}
		AccountId20::from(seed)
	}
}

impl<
		T: pallet_identity::Config<
			OffchainSignature = Signature,
			SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer,
		>,
	> pallet_identity::BenchmarkHelper<T::SigningPublicKey, T::OffchainSignature>
	for BenchmarkHelper<T>
{
	fn sign_message(message: &[u8]) -> (T::SigningPublicKey, T::OffchainSignature) {
		// Generate an ECDSA keypair using host functions (similar to default pallet_identity implementation)
		let public = sp_io::crypto::ecdsa_generate(0.into(), None);

		// Hash the message with keccak256 (Ethereum standard)
		let hash = sp_io::hashing::keccak_256(message);

		// Sign using the generated key
		let signature = sp_io::crypto::ecdsa_sign_prehashed(0.into(), &public, &hash)
			.expect("signing should succeed");

		// Convert to Ethereum types using existing From implementations
		let eth_signature = T::OffchainSignature::from(signature);
		let eth_signer = T::SigningPublicKey::from(public);

		(eth_signer, eth_signature)
	}
}
