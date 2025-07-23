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

//! # Ethereum Xcm pallet
//!
//! The Xcm Ethereum pallet is a bridge for Xcm Transact to Ethereum pallet

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::comparison_chain, clippy::large_enum_variant)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use ethereum_types::{H160, H256, U256};
use fp_ethereum::{TransactionData, ValidatedTransaction};
use fp_evm::{CheckEvmTransaction, CheckEvmTransactionConfig, TransactionValidationError};
use frame_support::{
	dispatch::{DispatchResultWithPostInfo, Pays, PostDispatchInfo},
	traits::{EnsureOrigin, Get, ProcessMessage},
	weights::Weight,
};
use frame_system::pallet_prelude::OriginFor;
use pallet_evm::{AddressMapping, GasWeightMapping};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::UniqueSaturatedInto, DispatchErrorWithPostInfo, RuntimeDebug};
use sp_std::{marker::PhantomData, prelude::*};

pub use ethereum::{
	AccessListItem, BlockV3 as Block, LegacyTransactionMessage, Log, ReceiptV4 as Receipt,
	TransactionAction, TransactionV3 as Transaction,
};
pub use fp_rpc::TransactionStatus;
pub use xcm_primitives::{EnsureProxy, EthereumXcmTransaction, XcmToEthereum};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RawOrigin {
	XcmEthereumTransaction(H160),
}

pub fn ensure_xcm_ethereum_transaction<OuterOrigin>(o: OuterOrigin) -> Result<H160, &'static str>
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
	match o.into() {
		Ok(RawOrigin::XcmEthereumTransaction(n)) => Ok(n),
		_ => Err("bad origin: expected to be a xcm Ethereum transaction"),
	}
}

pub struct EnsureXcmEthereumTransaction;
impl<O: Into<Result<RawOrigin, O>> + From<RawOrigin>> EnsureOrigin<O>
	for EnsureXcmEthereumTransaction
{
	type Success = H160;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().map(|o| match o {
			RawOrigin::XcmEthereumTransaction(id) => id,
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		Ok(O::from(RawOrigin::XcmEthereumTransaction(
			Default::default(),
		)))
	}
}

environmental::environmental!(XCM_MESSAGE_HASH: H256);

pub struct MessageProcessorWrapper<Inner>(core::marker::PhantomData<Inner>);
impl<Inner: ProcessMessage> ProcessMessage for MessageProcessorWrapper<Inner> {
	type Origin = <Inner as ProcessMessage>::Origin;

	fn process_message(
		message: &[u8],
		origin: Self::Origin,
		meter: &mut frame_support::weights::WeightMeter,
		id: &mut [u8; 32],
	) -> Result<bool, frame_support::traits::ProcessMessageError> {
		let mut xcm_msg_hash = H256(sp_io::hashing::blake2_256(message));
		XCM_MESSAGE_HASH::using(&mut xcm_msg_hash, || {
			Inner::process_message(message, origin, meter, id)
		})
	}
}

pub use self::pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use fp_evm::AccountProvider;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Invalid transaction error
		type InvalidEvmTransactionError: From<TransactionValidationError>;
		/// Handler for applying an already validated transaction
		type ValidatedTransaction: ValidatedTransaction;
		/// Origin for xcm transact
		type XcmEthereumOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = H160>;
		/// Maximum Weight reserved for xcm in a block
		type ReservedXcmpWeight: Get<Weight>;
		/// Ensure proxy
		type EnsureProxy: EnsureProxy<
			<<Self as pallet_evm::Config>::AccountProvider as AccountProvider>::AccountId,
		>;
		/// The origin that is allowed to resume or suspend the XCM to Ethereum executions.
		type ControllerOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// An origin that can submit a create tx type
		type ForceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Global nonce used for building Ethereum transaction payload.
	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	pub(crate) type Nonce<T: Config> = StorageValue<_, U256, ValueQuery>;

	/// Whether or not Ethereum-XCM is suspended from executing
	#[pallet::storage]
	#[pallet::getter(fn ethereum_xcm_suspended)]
	pub(super) type EthereumXcmSuspended<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::origin]
	pub type Origin = RawOrigin;

	#[pallet::error]
	pub enum Error<T> {
		/// Xcm to Ethereum execution is suspended
		EthereumXcmExecutionSuspended,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		/// Ethereum transaction executed from XCM
		ExecutedFromXcm {
			xcm_msg_hash: H256,
			eth_tx_hash: H256,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		OriginFor<T>: Into<Result<RawOrigin, OriginFor<T>>>,
	{
		/// Xcm Transact an Ethereum transaction.
		/// Weight: Gas limit plus the db read involving the suspension check
		#[pallet::weight({
			let without_base_extrinsic_weight = false;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				match xcm_transaction {
					EthereumXcmTransaction::V1(v1_tx) =>  v1_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V2(v2_tx) =>  v2_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V3(v3_tx) =>  v3_tx.gas_limit.unique_saturated_into(),
				}
			}, without_base_extrinsic_weight).saturating_add(T::DbWeight::get().reads(1))
		})]
		pub fn transact(
			origin: OriginFor<T>,
			xcm_transaction: EthereumXcmTransaction,
		) -> DispatchResultWithPostInfo {
			let source = T::XcmEthereumOrigin::ensure_origin(origin)?;
			ensure!(
				!EthereumXcmSuspended::<T>::get(),
				DispatchErrorWithPostInfo {
					error: Error::<T>::EthereumXcmExecutionSuspended.into(),
					post_info: PostDispatchInfo {
						actual_weight: Some(T::DbWeight::get().reads(1)),
						pays_fee: Pays::Yes
					}
				}
			);
			Self::validate_and_apply(source, xcm_transaction, false, None)
		}

		/// Xcm Transact an Ethereum transaction through proxy.
		/// Weight: Gas limit plus the db reads involving the suspension and proxy checks
		#[pallet::weight({
			let without_base_extrinsic_weight = false;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				match xcm_transaction {
					EthereumXcmTransaction::V1(v1_tx) =>  v1_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V2(v2_tx) =>  v2_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V3(v3_tx) =>  v3_tx.gas_limit.unique_saturated_into(),
				}
			}, without_base_extrinsic_weight).saturating_add(T::DbWeight::get().reads(2))
		})]
		pub fn transact_through_proxy(
			origin: OriginFor<T>,
			transact_as: H160,
			xcm_transaction: EthereumXcmTransaction,
		) -> DispatchResultWithPostInfo {
			let source = T::XcmEthereumOrigin::ensure_origin(origin)?;
			ensure!(
				!EthereumXcmSuspended::<T>::get(),
				DispatchErrorWithPostInfo {
					error: Error::<T>::EthereumXcmExecutionSuspended.into(),
					post_info: PostDispatchInfo {
						actual_weight: Some(T::DbWeight::get().reads(1)),
						pays_fee: Pays::Yes
					}
				}
			);
			let _ = T::EnsureProxy::ensure_ok(
				T::AddressMapping::into_account_id(transact_as),
				T::AddressMapping::into_account_id(source),
			)
			.map_err(|e| sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(T::DbWeight::get().reads(2)),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other(e),
			})?;

			Self::validate_and_apply(transact_as, xcm_transaction, false, None)
		}

		/// Suspends all Ethereum executions from XCM.
		///
		/// - `origin`: Must pass `ControllerOrigin`.
		#[pallet::weight((T::DbWeight::get().writes(1), DispatchClass::Operational,))]
		pub fn suspend_ethereum_xcm_execution(origin: OriginFor<T>) -> DispatchResult {
			T::ControllerOrigin::ensure_origin(origin)?;

			EthereumXcmSuspended::<T>::put(true);

			Ok(())
		}

		/// Resumes all Ethereum executions from XCM.
		///
		/// - `origin`: Must pass `ControllerOrigin`.
		#[pallet::weight((T::DbWeight::get().writes(1), DispatchClass::Operational,))]
		pub fn resume_ethereum_xcm_execution(origin: OriginFor<T>) -> DispatchResult {
			T::ControllerOrigin::ensure_origin(origin)?;

			EthereumXcmSuspended::<T>::put(false);

			Ok(())
		}

		/// Xcm Transact an Ethereum transaction, but allow to force the caller and create address.
		/// This call should be restricted (callable only by the runtime or governance).
		/// Weight: Gas limit plus the db reads involving the suspension and proxy checks
		#[pallet::weight({
			let without_base_extrinsic_weight = false;
			<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight({
				match xcm_transaction {
					EthereumXcmTransaction::V1(v1_tx) => v1_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V2(v2_tx) => v2_tx.gas_limit.unique_saturated_into(),
					EthereumXcmTransaction::V3(v3_tx) => v3_tx.gas_limit.unique_saturated_into(),
				}
			}, without_base_extrinsic_weight).saturating_add(T::DbWeight::get().reads(1))
		})]
		pub fn force_transact_as(
			origin: OriginFor<T>,
			transact_as: H160,
			xcm_transaction: EthereumXcmTransaction,
			force_create_address: Option<H160>,
		) -> DispatchResultWithPostInfo {
			T::ForceOrigin::ensure_origin(origin)?;
			ensure!(
				!EthereumXcmSuspended::<T>::get(),
				DispatchErrorWithPostInfo {
					error: Error::<T>::EthereumXcmExecutionSuspended.into(),
					post_info: PostDispatchInfo {
						actual_weight: Some(T::DbWeight::get().reads(1)),
						pays_fee: Pays::Yes
					}
				}
			);

			Self::validate_and_apply(transact_as, xcm_transaction, true, force_create_address)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn transaction_len(transaction: &Transaction) -> u64 {
		transaction
			.encode()
			.len()
			// pallet + call indexes
			.saturating_add(2) as u64
	}

	fn validate_and_apply(
		source: H160,
		xcm_transaction: EthereumXcmTransaction,
		allow_create: bool,
		maybe_force_create_address: Option<H160>,
	) -> DispatchResultWithPostInfo {
		// The lack of a real signature where different callers with the
		// same nonce are providing identical transaction payloads results in a collision and
		// the same ethereum tx hash.
		// We use a global nonce instead the user nonce for all Xcm->Ethereum transactions to avoid
		// this.
		let current_nonce = Self::nonce();
		let error_weight = T::DbWeight::get().reads(1);

		let transaction: Option<Transaction> =
			xcm_transaction.into_transaction(current_nonce, T::ChainId::get(), allow_create);
		if let Some(transaction) = transaction {
			let tx_hash = transaction.hash();
			let transaction_data: TransactionData = (&transaction).into();

			let (weight_limit, proof_size_base_cost) =
				match <T as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					transaction_data.gas_limit.unique_saturated_into(),
					true,
				) {
					weight_limit if weight_limit.proof_size() > 0 => (
						Some(weight_limit),
						Some(Self::transaction_len(&transaction)),
					),
					_ => (None, None),
				};

			let _ = CheckEvmTransaction::<T::InvalidEvmTransactionError>::new(
				CheckEvmTransactionConfig {
					evm_config: T::config(),
					block_gas_limit: U256::from(
						<T as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
							T::ReservedXcmpWeight::get(),
						),
					),
					base_fee: U256::zero(),
					chain_id: 0u64,
					is_transactional: true,
				},
				transaction_data.into(),
				weight_limit,
				proof_size_base_cost,
			)
			// We only validate the gas limit against the evm transaction cost.
			// No need to validate fee payment, as it is handled by the xcm executor.
			.validate_common()
			.map_err(|_| sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(error_weight),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other("Failed to validate ethereum transaction"),
			})?;

			// Once we know a new transaction hash exists - the user can afford storing the
			// transaction on chain - we increase the global nonce.
			<Nonce<T>>::put(current_nonce.saturating_add(U256::one()));

			let (dispatch_info, _) =
				T::ValidatedTransaction::apply(source, transaction, maybe_force_create_address)?;

			XCM_MESSAGE_HASH::with(|xcm_msg_hash| {
				Self::deposit_event(Event::ExecutedFromXcm {
					xcm_msg_hash: *xcm_msg_hash,
					eth_tx_hash: tx_hash,
				});
			});

			Ok(dispatch_info)
		} else {
			Err(sp_runtime::DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(error_weight),
					pays_fee: Pays::Yes,
				},
				error: sp_runtime::DispatchError::Other("Cannot convert xcm payload to known type"),
			})
		}
	}
}
