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
	BalanceOf, Config, Error, Event, GetBabeData, Pallet, RandomnessResults, RelayTime, RequestId,
};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive};
use pallet_evm::AddressMapping;
use sp_core::{H160, H256};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Type of request
/// Represents a request for the most recent randomness of this type at or after the inner time
pub enum RequestType<T: Config> {
	/// Babe per relay chain block
	BabeCurrentBlock(T::BlockNumber),
	/// Babe one epoch ago
	BabeOneEpochAgo(u64),
	/// Babe two epochs ago
	BabeTwoEpochsAgo(u64),
	/// Local per parachain block VRF output
	Local(T::BlockNumber),
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Raw randomness snapshot, the unique value for a `RequestType` in `RandomnessResults` map
pub struct RandomnessResult<Hash> {
	/// Randomness once available
	pub randomness: Option<Hash>,
	/// Number of randomness requests for the type
	pub request_count: u64,
}

impl<Hash: Clone> RandomnessResult<Hash> {
	pub fn new() -> RandomnessResult<Hash> {
		RandomnessResult {
			randomness: None,
			request_count: 1u64,
		}
	}
	/// Increment request count
	pub fn increment_request_count<T: Config>(&self) -> Result<Self, DispatchError> {
		let new_request_count = self
			.request_count
			.checked_add(1u64)
			.ok_or(Error::<T>::RequestCounterOverflowed)?;
		Ok(RandomnessResult {
			randomness: self.randomness.clone(),
			request_count: new_request_count,
		})
	}
	/// Returns whether successfully decremented
	/// Failure implies the randomness result should be removed from storage
	pub fn decrement_request_count(&self) -> Option<Self> {
		if let Some(new_request_count) = self.request_count.checked_sub(1u64) {
			Some(RandomnessResult {
				randomness: self.randomness.clone(),
				request_count: new_request_count,
			})
		} else {
			None
		}
	}
}

#[derive(Default, PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Relay time information
pub struct RelayTimeInfo<BlockNumber, EpochIndex> {
	pub relay_block_number: BlockNumber,
	pub relay_epoch_index: EpochIndex,
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Input arguments to request randomness
pub struct Request<T: Config> {
	/// Fee is returned to this account upon execution
	pub refund_address: H160,
	/// Contract that consumes the randomness
	pub contract_address: H160,
	/// Fee to pay for execution
	pub fee: BalanceOf<T>,
	/// Gas limit for subcall
	pub gas_limit: u64,
	/// Salt to use once randomness is ready
	pub salt: H256,
	/// Details regarding request type
	pub info: RequestType<T>,
}

impl<T: Config> Request<T> {
	pub fn validate(&self) -> Result<Expiration<T::BlockNumber>, DispatchError> {
		ensure!(
			!self.can_be_fulfilled(),
			Error::<T>::CannotRequestPastRandomness
		);
		let (due_before_expiry, expires) = match self.info {
			RequestType::BabeCurrentBlock(block) => {
				// assumes 1 relay per para block
				let expiring_relay_block = RelayTime::<T>::get()
					.relay_block_number
					.saturating_add(T::ExpirationDelay::get().into());
				(
					block < expiring_relay_block,
					Expiration::Block(expiring_relay_block),
				)
			}
			RequestType::BabeOneEpochAgo(epoch) => {
				let expiring_relay_epoch_index = RelayTime::<T>::get()
					.relay_epoch_index
					.saturating_add(T::ExpirationDelay::get().into());
				(
					epoch < expiring_relay_epoch_index,
					Expiration::Epoch(expiring_relay_epoch_index),
				)
			}
			RequestType::BabeTwoEpochsAgo(epoch) => {
				let expiring_relay_epoch_index = RelayTime::<T>::get()
					.relay_epoch_index
					.saturating_add(T::ExpirationDelay::get().into());
				(
					epoch < expiring_relay_epoch_index,
					Expiration::Epoch(expiring_relay_epoch_index),
				)
			}
			RequestType::Local(block) => {
				let expires = frame_system::Pallet::<T>::block_number()
					.saturating_add(T::ExpirationDelay::get().into());
				(block < expires, Expiration::Block(expires))
			}
		};
		ensure!(
			due_before_expiry,
			Error::<T>::CannotRequestRandomnessAfterExpirationDelay
		);
		Ok(expires)
	}
	pub fn can_be_fulfilled(&self) -> bool {
		match self.info {
			RequestType::BabeCurrentBlock(block) => {
				block <= T::BabeDataGetter::get_relay_block_number()
			}
			RequestType::BabeOneEpochAgo(epoch) => {
				epoch <= T::BabeDataGetter::get_relay_epoch_index()
			}
			RequestType::BabeTwoEpochsAgo(epoch) => {
				epoch <= T::BabeDataGetter::get_relay_epoch_index()
			}
			RequestType::Local(block) => block <= frame_system::Pallet::<T>::block_number(),
		}
	}
	fn get_randomness(&self) -> Result<T::Hash, DispatchError> {
		ensure!(
			self.can_be_fulfilled(),
			Error::<T>::RequestCannotYetBeFulfilled
		);
		let randomness = <RandomnessResults<T>>::get(&self.info)
			// hitting this error is a bug because a RandomnessResult should exist if request exists
			.ok_or(Error::<T>::RandomnessResultDNE)?
			.randomness
			// hitting this error is a bug because a RandomnessResult should be updated if request
			// can be fulfilled
			.ok_or(Error::<T>::RandomnessResultNotFilled)?;
		Ok(randomness)
	}
	pub(crate) fn emit_randomness_requested_event(&self, id: RequestId) {
		let event = match self.info {
			RequestType::BabeCurrentBlock(block) => Event::<T>::RandomnessRequestedCurrentBlock {
				id,
				refund_address: self.refund_address.clone(),
				contract_address: self.contract_address.clone(),
				fee: self.fee,
				gas_limit: self.gas_limit,
				salt: self.salt,
				earliest_block: block,
			},
			RequestType::BabeOneEpochAgo(index) => Event::<T>::RandomnessRequestedBabeOneEpochAgo {
				id,
				refund_address: self.refund_address.clone(),
				contract_address: self.contract_address.clone(),
				fee: self.fee,
				gas_limit: self.gas_limit,
				salt: self.salt,
				earliest_epoch: index,
			},
			RequestType::BabeTwoEpochsAgo(index) => {
				Event::<T>::RandomnessRequestedBabeTwoEpochsAgo {
					id,
					refund_address: self.refund_address.clone(),
					contract_address: self.contract_address.clone(),
					fee: self.fee,
					gas_limit: self.gas_limit,
					salt: self.salt,
					earliest_epoch: index,
				}
			}
			RequestType::Local(block) => Event::<T>::RandomnessRequestedLocal {
				id,
				refund_address: self.refund_address.clone(),
				contract_address: self.contract_address.clone(),
				fee: self.fee,
				gas_limit: self.gas_limit,
				salt: self.salt,
				earliest_block: block,
			},
		};
		Pallet::<T>::deposit_event(event);
	}
	/// Cleanup after fulfilling a request
	pub(crate) fn finish_fulfill(
		&self,
		deposit: BalanceOf<T>,
		caller: &H160,
		cost_of_execution: BalanceOf<T>,
	) {
		let refundable_amount = deposit.saturating_add(self.fee);
		if let Some(excess) = refundable_amount.checked_sub(&cost_of_execution) {
			// send excess to refund address
			T::Currency::transfer(
				&Pallet::<T>::account_id(),
				&T::AddressMapping::into_account_id(self.refund_address),
				excess,
				KeepAlive,
			)
			.expect("excess should be transferrable");
		}
		// refund cost_of_execution to caller of `fulfill`
		T::Currency::transfer(
			&Pallet::<T>::account_id(),
			&T::AddressMapping::into_account_id(caller.clone()),
			cost_of_execution,
			KeepAlive,
		)
		.expect("cost_of_execution should be transferrable");
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Expiration<BlockNumber> {
	Block(BlockNumber),
	Epoch(u64),
}

impl<B: Copy> Expiration<B> {
	fn epoch(&self) -> u64 {
		match self {
			Expiration::Block(_) => panic!("Cannot access epoch when expiration is block type"),
			Expiration::Epoch(epoch) => *epoch,
		}
	}
	fn block(&self) -> B {
		match self {
			Expiration::Block(block) => *block,
			Expiration::Epoch(_) => panic!("Cannot access block when expiration is epoch type"),
		}
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RequestState<T: Config> {
	/// Underlying request
	pub request: Request<T>,
	/// Deposit taken for making request (stored in case config changes)
	pub deposit: BalanceOf<T>,
	/// Expiration block or epoch index depending on request type
	pub expires: Expiration<T::BlockNumber>,
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Data required to make the subcallback and finish fulfilling the request
pub struct FulfillArgs<T: Config> {
	/// Original request
	pub request: Request<T>,
	/// Deposit for request
	pub deposit: BalanceOf<T>,
	/// Randomness
	pub randomness: [u8; 32],
}

impl<T: Config> RequestState<T> {
	pub(crate) fn new(request: Request<T>) -> Result<RequestState<T>, DispatchError> {
		let expires = request.validate()?;
		Ok(RequestState {
			request,
			deposit: T::Deposit::get(),
			expires,
		})
	}
	/// Returns Ok(FulfillArgs) if successful
	/// This should be called before the callback
	pub fn prepare_fulfill(&self) -> Result<FulfillArgs<T>, DispatchError> {
		// get the randomness corresponding to the request
		let randomness: T::Hash = self.request.get_randomness()?;
		// compute random output using salt
		let randomness = Pallet::<T>::concat_and_hash(randomness, self.request.salt);
		// No event emitted until fulfillment is complete
		Ok(FulfillArgs {
			request: self.request.clone(),
			deposit: self.deposit,
			randomness,
		})
	}
	pub fn increase_fee(
		&mut self,
		caller: &H160,
		fee_increase: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		ensure!(
			caller == &self.request.contract_address,
			Error::<T>::OnlyRequesterCanIncreaseFee
		);
		let new_fee = self
			.request
			.fee
			.checked_add(&fee_increase)
			.ok_or(Error::<T>::RequestFeeOverflowed)?;
		let caller = T::AddressMapping::into_account_id(caller.clone());
		T::Currency::transfer(&caller, &Pallet::<T>::account_id(), fee_increase, KeepAlive)?;
		self.request.fee = new_fee;
		Ok(new_fee)
	}
	/// Transfer deposit back to contract_address
	/// Transfer fee to caller
	pub fn execute_expiration(&self, caller: &T::AccountId) -> DispatchResult {
		let request_has_expired = match self.request.info {
			RequestType::BabeCurrentBlock(_) => {
				RelayTime::<T>::get().relay_block_number >= self.expires.block()
			}
			RequestType::BabeOneEpochAgo(_) => {
				RelayTime::<T>::get().relay_epoch_index >= self.expires.epoch()
			}
			RequestType::BabeTwoEpochsAgo(_) => {
				RelayTime::<T>::get().relay_epoch_index >= self.expires.epoch()
			}
			RequestType::Local(_) => {
				frame_system::Pallet::<T>::block_number() >= self.expires.block()
			}
		};
		ensure!(request_has_expired, Error::<T>::RequestHasNotExpired);
		let contract_address =
			T::AddressMapping::into_account_id(self.request.contract_address.clone());
		// TODO: is it worth optimizing when caller == contract_address to do one transfer here
		T::Currency::transfer(
			&Pallet::<T>::account_id(),
			&contract_address,
			self.deposit,
			KeepAlive,
		)
		.expect("expect transferrable deposit + fee, transferring deposit");
		T::Currency::transfer(
			&Pallet::<T>::account_id(),
			caller,
			self.request.fee,
			KeepAlive,
		)
		.expect("expect transferrable deposit + fee, transferring fee");
		Ok(())
	}
}
