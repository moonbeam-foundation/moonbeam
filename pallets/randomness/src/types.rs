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

use crate::{BalanceOf, Config, Error, Event, Pallet, RandomnessResults, RelayEpoch, RequestId};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive};
use pallet_evm::AddressMapping;
use sp_core::{H160, H256};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating};
use sp_std::vec::Vec;

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Shared request info, a subset of `RequestInfo`
pub enum RequestType<T: Config> {
	/// Babe one epoch ago
	BabeEpoch(u64),
	/// Local per parachain block VRF output
	Local(T::BlockNumber),
}

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Type of request
/// Represents a request for the most recent randomness at or after the inner first field
/// Expiration is second inner field
pub enum RequestInfo<T: Config> {
	/// Babe one epoch ago
	BabeEpoch(u64, u64),
	/// Local per parachain block VRF output
	Local(T::BlockNumber, T::BlockNumber),
}

impl<T: Config> From<RequestType<T>> for RequestInfo<T> {
	fn from(other: RequestType<T>) -> RequestInfo<T> {
		// add expiration
		match other {
			RequestType::BabeEpoch(epoch) => RequestInfo::BabeEpoch(
				epoch,
				RelayEpoch::<T>::get().saturating_add(T::EpochExpirationDelay::get()),
			),
			RequestType::Local(block) => RequestInfo::Local(
				block,
				frame_system::Pallet::<T>::block_number()
					.saturating_add(T::BlockExpirationDelay::get()),
			),
		}
	}
}

impl<T: Config> From<RequestInfo<T>> for RequestType<T> {
	fn from(other: RequestInfo<T>) -> RequestType<T> {
		match other {
			RequestInfo::BabeEpoch(epoch, _) => RequestType::BabeEpoch(epoch),
			RequestInfo::Local(block, _) => RequestType::Local(block),
		}
	}
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
	pub fn increment_request_count(&self) -> Self {
		let new_request_count = self.request_count.saturating_add(1u64);
		RandomnessResult {
			randomness: self.randomness.clone(),
			request_count: new_request_count,
		}
	}
	/// Returns whether successfully decremented
	/// Failure implies the randomness result should be removed from storage
	pub fn decrement_request_count(&self) -> Option<Self> {
		if let Some(new_request_count) = self.request_count.checked_sub(1u64) {
			if new_request_count == 0u64 {
				return None;
			}
			Some(RandomnessResult {
				randomness: self.randomness.clone(),
				request_count: new_request_count,
			})
		} else {
			None
		}
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Input arguments to request randomness
pub struct Request<Balance, Info> {
	/// Fee is returned to this account upon execution
	pub refund_address: H160,
	/// Contract that consumes the randomness
	pub contract_address: H160,
	/// Fee to pay for execution
	pub fee: Balance,
	/// Gas limit for subcall
	pub gas_limit: u64,
	/// Number of random outputs requested
	pub num_words: u8,
	/// Salt to use once randomness is ready
	pub salt: H256,
	/// Details regarding request type
	pub info: Info,
}

impl<T: Config> From<Request<BalanceOf<T>, RequestType<T>>>
	for Request<BalanceOf<T>, RequestInfo<T>>
{
	fn from(other: Request<BalanceOf<T>, RequestType<T>>) -> Request<BalanceOf<T>, RequestInfo<T>> {
		Request {
			refund_address: other.refund_address,
			contract_address: other.contract_address,
			fee: other.fee,
			gas_limit: other.gas_limit,
			num_words: other.num_words,
			salt: other.salt,
			info: other.info.into(),
		}
	}
}

impl<T: Config> Request<BalanceOf<T>, RequestInfo<T>> {
	pub fn is_expired(&self) -> bool {
		match self.info {
			RequestInfo::BabeEpoch(_, expires) => RelayEpoch::<T>::get() >= expires,
			RequestInfo::Local(_, expires) => frame_system::Pallet::<T>::block_number() >= expires,
		}
	}
	pub fn can_be_fulfilled(&self) -> bool {
		match self.info {
			RequestInfo::BabeEpoch(epoch_due, _) => epoch_due <= RelayEpoch::<T>::get(),
			RequestInfo::Local(block_due, _) => {
				block_due <= frame_system::Pallet::<T>::block_number()
			}
		}
	}
	pub fn validate(&mut self) -> DispatchResult {
		ensure!(
			self.num_words <= T::MaxRandomWords::get(),
			Error::<T>::CannotRequestMoreWordsThanMax
		);
		ensure!(self.num_words >= 1u8, Error::<T>::MustRequestAtLeastOneWord);
		match self.info {
			RequestInfo::Local(block_due, _) => {
				let current_block = frame_system::Pallet::<T>::block_number();
				ensure!(
					block_due
						<= current_block
							.checked_add(&T::MaxBlockDelay::get())
							.ok_or(Error::<T>::CannotRequestRandomnessAfterMaxDelay)?,
					Error::<T>::CannotRequestRandomnessAfterMaxDelay
				);
				ensure!(
					block_due
						>= current_block
							.checked_add(&T::MinBlockDelay::get())
							.ok_or(Error::<T>::CannotRequestRandomnessBeforeMinDelay)?,
					Error::<T>::CannotRequestRandomnessBeforeMinDelay
				);
			}
			_ => (), // not necessary because epoch delay is not an input to precompile
		}
		Ok(())
	}
	fn get_random_words(&self) -> Result<Vec<[u8; 32]>, DispatchError> {
		ensure!(
			self.can_be_fulfilled(),
			Error::<T>::RequestCannotYetBeFulfilled
		);
		let info_key: RequestType<T> = self.info.clone().into();
		let randomness = <RandomnessResults<T>>::get(&info_key)
			// hitting this error is a bug because a RandomnessResult should exist if request exists
			.ok_or(Error::<T>::RandomnessResultDNE)?
			.randomness
			// hitting this error is a bug because a RandomnessResult should be updated if request
			// can be fulfilled
			.ok_or(Error::<T>::RandomnessResultNotFilled)?;
		// Returns Vec<[u8; 32]> of length `num_words`
		// Each element is the blake2_256 of the concatenation of `randomness + salt + i` such that
		// `0<=i<num_words`.
		let compute_random_words = |random: T::Hash, salt: H256, num_words| {
			let mut output: Vec<[u8; 32]> = Vec::new();
			let mut word = Vec::new();
			for index in 0u8..num_words {
				word.extend_from_slice(random.as_ref());
				word.extend_from_slice(salt.as_ref());
				word.extend_from_slice(&[index]);
				output.push(sp_io::hashing::blake2_256(&word));
				word.clear();
			}
			output
		};
		// return random words
		Ok(compute_random_words(randomness, self.salt, self.num_words))
	}
	pub(crate) fn emit_randomness_requested_event(&self, id: RequestId) {
		let event = match self.info {
			RequestInfo::BabeEpoch(index, _) => Event::<T>::RandomnessRequestedBabeEpoch {
				id,
				refund_address: self.refund_address.clone(),
				contract_address: self.contract_address.clone(),
				fee: self.fee,
				gas_limit: self.gas_limit,
				num_words: self.num_words,
				salt: self.salt,
				earliest_epoch: index,
			},
			RequestInfo::Local(block, _) => Event::<T>::RandomnessRequestedLocal {
				id,
				refund_address: self.refund_address.clone(),
				contract_address: self.contract_address.clone(),
				fee: self.fee,
				gas_limit: self.gas_limit,
				num_words: self.num_words,
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
		let try_transfer_or_log_error =
			|from: &T::AccountId, to: &T::AccountId, amount: BalanceOf<T>| {
				if let Err(error) = T::Currency::transfer(from, to, amount, KeepAlive) {
					log::warn!(
						"Failed to transfer in finish_fulfill with error {:?}",
						error,
					);
				}
			};
		let refundable_amount = deposit.saturating_add(self.fee);
		if let Some(excess) = refundable_amount.checked_sub(&cost_of_execution) {
			if &self.refund_address == caller {
				// send excess + cost of execution to refund address iff refund address is caller
				try_transfer_or_log_error(
					&Pallet::<T>::account_id(),
					&T::AddressMapping::into_account_id(self.refund_address),
					excess.saturating_add(cost_of_execution),
				);
				return;
			}
			// send excess to refund address
			try_transfer_or_log_error(
				&Pallet::<T>::account_id(),
				&T::AddressMapping::into_account_id(self.refund_address),
				excess,
			);
		}
		// refund cost_of_execution to caller of `fulfill`
		try_transfer_or_log_error(
			&Pallet::<T>::account_id(),
			&T::AddressMapping::into_account_id(caller.clone()),
			cost_of_execution,
		);
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RequestState<T: Config> {
	/// Underlying request
	pub request: Request<BalanceOf<T>, RequestInfo<T>>,
	/// Deposit taken for making request (stored in case config changes)
	pub deposit: BalanceOf<T>,
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Data required to make the subcallback and finish fulfilling the request
pub struct FulfillArgs<T: Config> {
	/// Original request
	pub request: Request<BalanceOf<T>, RequestInfo<T>>,
	/// Deposit for request
	pub deposit: BalanceOf<T>,
	/// Randomness
	pub randomness: Vec<[u8; 32]>,
}

impl<T: Config> RequestState<T> {
	pub(crate) fn new(
		mut request: Request<BalanceOf<T>, RequestInfo<T>>,
	) -> Result<RequestState<T>, DispatchError> {
		request.validate()?;
		Ok(RequestState {
			request,
			deposit: T::Deposit::get(),
		})
	}
	/// Returns Ok(FulfillArgs) if successful
	/// This should be called before the callback
	pub fn prepare_fulfill(&self) -> Result<FulfillArgs<T>, DispatchError> {
		// get the random words corresponding to the request
		let randomness = self.request.get_random_words()?;
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
		ensure!(self.request.is_expired(), Error::<T>::RequestHasNotExpired);
		let contract_address =
			T::AddressMapping::into_account_id(self.request.contract_address.clone());
		if caller == &contract_address {
			// If caller == contract_address, then transfer deposit + fee to contract_address
			T::Currency::transfer(
				&Pallet::<T>::account_id(),
				&contract_address,
				self.deposit.saturating_add(self.request.fee),
				KeepAlive,
			)?;
		} else {
			// Return deposit to `contract_address`
			T::Currency::transfer(
				&Pallet::<T>::account_id(),
				&contract_address,
				self.deposit,
				KeepAlive,
			)?;
			// Return request.fee to `caller`
			T::Currency::transfer(
				&Pallet::<T>::account_id(),
				caller,
				self.request.fee,
				KeepAlive,
			)?;
		}
		Ok(())
	}
}
