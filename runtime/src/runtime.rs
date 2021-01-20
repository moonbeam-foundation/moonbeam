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

//! Exposes the core runtime types.
//! Feature `standalone` allows to configure the runtime for a standalone node instead of a parachain.

use crate::{common::*, ethereum::EthereumFindAuthor, opaque, Box, Vec};
use fp_rpc::TransactionStatus;
use frame_support::{
	construct_runtime,
	traits::{FindAuthor, Get, Randomness},
	ConsensusEngineId,
};
use pallet_evm::{Account as EVMAccount, FeeCalculator, Runner};
use sp_api::impl_runtime_apis;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str,
	traits::Block as BlockT,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllModules,
>;

#[cfg(not(feature = "standalone"))]
pub use parachain::*;
#[cfg(feature = "standalone")]
pub use standalone::*;

#[cfg(not(feature = "standalone"))]
pub const NAME: &str = "moonbase-alphanet";
#[cfg(feature = "standalone")]
pub const NAME: &str = "moonbase-standalone";

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!(NAME),
	impl_name: create_runtime_str!(NAME),
	authoring_version: 3,
	spec_version: 9,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
};

#[cfg(not(feature = "standalone"))]
mod parachain {
	use super::*;

	impl cumulus_parachain_upgrade::Config for Runtime {
		type Event = Event;
		type OnValidationData = ();
		type SelfParaId = ParachainInfo;
	}

	impl parachain_info::Config for Runtime {}

	// TODO Consensus not supported in parachain.
	impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
		fn find_author<'a, I>(_digests: I) -> Option<H160>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			None
		}
	}

	/// Temporary dummy Aura implementation.
	pub struct PhantomAura;
	impl FindAuthor<u32> for PhantomAura {
		fn find_author<'a, I>(_digests: I) -> Option<u32>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			Some(0 as u32)
		}
	}
}

#[cfg(feature = "standalone")]
mod standalone {
	use super::*;
	pub use frame_support::traits::KeyOwnerProofSystem;
	pub use pallet_grandpa::{
		fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
	};
	pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
	pub use sp_core::crypto::{KeyTypeId, Public};
	pub use sp_runtime::traits::NumberFor;

	impl pallet_aura::Config for Runtime {
		type AuthorityId = AuraId;
	}

	impl pallet_grandpa::Config for Runtime {
		type Event = Event;
		type Call = Call;

		type KeyOwnerProofSystem = ();

		type KeyOwnerProof =
			<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

		type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
			KeyTypeId,
			GrandpaId,
		)>>::IdentificationTuple;

		type HandleEquivocation = ();
		type WeightInfo = ();
	}

	impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
		fn find_author<'a, I>(digests: I) -> Option<H160>
		where
			I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
		{
			if let Some(author_index) = F::find_author(digests) {
				let authority_id = Aura::authorities()[author_index as usize].clone();
				return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
			}
			None
		}
	}
}

#[cfg(not(feature = "standalone"))]
construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		AuthorInherent: author_inherent::{Call, Event<T>, Inherent, Module, Storage},
		Balances: pallet_balances::{Call, Config<T>, Event<T>, Module, Storage},
		Ethereum: pallet_ethereum::{Call, Config, Event, Module, Storage, ValidateUnsigned},
		EthereumChainId: pallet_ethereum_chain_id::{Config, Module, Storage},
		EVM: pallet_evm::{Call, Config, Event<T>, Module, Storage},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Call, Module, Storage},
		Stake: stake::{Call, Config<T>, Event<T>, Module, Storage},
		Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Module, Storage},
		System: frame_system::{Call, Config, Event<T>, Module, Storage},
		Timestamp: pallet_timestamp::{Call, Inherent, Module, Storage},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},

		ParachainInfo: parachain_info::{Config, Module, Storage},
		ParachainUpgrade: cumulus_parachain_upgrade::{Call, Event, Inherent, Module, Storage},
	}
}

#[cfg(feature = "standalone")]
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		AuthorInherent: author_inherent::{Call, Event<T>, Inherent, Module, Storage},
		Balances: pallet_balances::{Call, Config<T>, Event<T>, Module, Storage},
		Ethereum: pallet_ethereum::{Call, Config, Event, Module, Storage, ValidateUnsigned},
		EthereumChainId: pallet_ethereum_chain_id::{Config, Module, Storage},
		EVM: pallet_evm::{Call, Config, Event<T>, Module, Storage},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Call, Module, Storage},
		Stake: stake::{Call, Config<T>, Event<T>, Module, Storage},
		Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Module, Storage},
		System: frame_system::{Call, Config, Event<T>, Module, Storage},
		Timestamp: pallet_timestamp::{Call, Inherent, Module, Storage},
		TransactionPayment: pallet_transaction_payment::{Module, Storage},

		Aura: pallet_aura::{Config<T>, Inherent, Module},
		Grandpa: pallet_grandpa::{Call, Config, Event, Module, Storage},
	}
);

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(
			data: sp_inherents::InherentData
		) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			EVM::account_basic(&address)
		}

		fn gas_price() -> U256 {
			<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			EVM::account_codes(address)
		}

		fn author() -> H160 {
			<pallet_ethereum::Module<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			EVM::account_storages(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.low_u32(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			#[allow(clippy::or_fun_call)] // suggestion not helpful here
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.low_u32(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			Ethereum::current_transaction_statuses()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			Ethereum::current_block()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			Ethereum::current_receipts()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				Ethereum::current_block(),
				Ethereum::current_receipts(),
				Ethereum::current_transaction_statuses()
			)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		for Runtime {

		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
	}

	#[cfg(feature = "standalone")]
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	#[cfg(feature = "standalone")]
	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}
}

#[cfg(not(feature = "standalone"))]
cumulus_runtime::register_validate_block!(Block, Executive);
