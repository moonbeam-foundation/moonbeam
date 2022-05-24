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
	BalanceOf, Config, CurrentBlockRandomness, CurrentEpochIndex, Error, OneEpochAgoRandomness,
	TwoEpochsAgoRandomness,
};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency};
use pallet_vrf::GetMaybeRandomness;
use sp_runtime::traits::{CheckedSub, Saturating};

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Type of request
/// Represents a request for the most recent randomness of this type at or after the inner time
pub enum RequestType<T: Config> {
	/// Babe one epoch ago
	BabeOneEpochAgo(u64),
	/// Babe two epochs ago
	BabeTwoEpochsAgo(u64),
	/// Babe per block
	BabeCurrentBlock(T::BlockNumber),
	/// Local per block VRF output
	Local(T::BlockNumber),
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Input arguments to request randomness
pub struct Request<T: Config> {
	/// Fee is returned to this account upon execution
	pub refund_address: T::AccountId,
	/// Contract that consumes the randomness
	pub contract_address: T::AccountId,
	/// Fee to pay for execution
	pub fee: BalanceOf<T>,
	/// Salt to use once randomness is ready
	pub salt: T::Hash,
	/// Details regarding request type
	pub info: RequestType<T>,
}

impl<T: Config> Request<T> {
	pub fn can_be_fulfilled(&self) -> bool {
		let leq_current_block =
			|when| -> bool { when <= frame_system::Pallet::<T>::block_number() };
		let leq_current_epoch_index = |when| -> bool { when <= <CurrentEpochIndex<T>>::get() };
		match self.info {
			RequestType::BabeOneEpochAgo(index) => leq_current_epoch_index(index),
			RequestType::BabeTwoEpochsAgo(index) => leq_current_epoch_index(index),
			RequestType::BabeCurrentBlock(block) => leq_current_block(block),
			RequestType::Local(block) => leq_current_block(block),
		}
	}
	/// Cleanup after fulfilling a request
	/// This assumes the request was fulfilled and the callback did NOT OOG
	pub(crate) fn finish_fulfill(
		&self,
		deposit: BalanceOf<T>,
		caller: &T::AccountId,
		cost_of_execution: BalanceOf<T>,
		excess: BalanceOf<T>,
	) -> DispatchResult {
		T::Currency::unreserve(&self.contract_address, deposit + self.fee);
		// refund cost_of_execution to caller
		T::Currency::transfer(&self.contract_address, caller, cost_of_execution, KeepAlive)
			.expect("just unreserved deposit + fee => cost_of_execution must be transferrable");
		// refund excess fee to refund address
		T::Currency::transfer(
			&self.contract_address,
			&self.refund_address,
			excess,
			KeepAlive,
		)
		.expect("just unreserved deposit + fee => excess must be transferrable");
		Ok(())
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RequestState<T: Config> {
	/// Underlying request
	pub request: Request<T>,
	/// Deposit taken for making request (stored in case config changes)
	pub deposit: BalanceOf<T>,
	/// All requests expire `T::ExpirationDelay` blocks after they are made
	pub expires: T::BlockNumber,
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
/// Data required to make the subcallback and finish fulfilling the request
pub struct FulfillArgs<T: Config> {
	/// Original request
	pub request: Request<T>,
	/// Deposit for request
	pub deposit: BalanceOf<T>,
	/// Contract that consumes the randomness
	pub contract_address: T::AccountId,
	/// Gas limit for subcall, requires conversion to u64
	/// by multiplying by BaseFee
	pub gas_limit: BalanceOf<T>,
	/// Randomness
	pub randomness: T::Hash,
}

impl<T: Config> RequestState<T> {
	pub(crate) fn new(request: Request<T>, deposit: BalanceOf<T>) -> RequestState<T> {
		let expires =
			frame_system::Pallet::<T>::block_number().saturating_add(T::ExpirationDelay::get());
		RequestState {
			request,
			deposit,
			expires,
		}
	}
	/// Get the randomness corresponding to the request
	/// Only fails if the randomness is not available
	/// Only should be called in `prepare_fulfill` after check that the request can be fulfilled
	fn get_randomness(&self) -> Result<T::Hash, DispatchError> {
		match self.request.info {
			RequestType::BabeOneEpochAgo(_) => OneEpochAgoRandomness::<T>::get(),
			RequestType::BabeTwoEpochsAgo(_) => TwoEpochsAgoRandomness::<T>::get(),
			RequestType::BabeCurrentBlock(_) => CurrentBlockRandomness::<T>::get(),
			RequestType::Local(_) => T::LocalRandomness::get_current_randomness(),
		}
		.ok_or(Error::<T>::RandomnessNotAvailable.into())
	}
	/// Returns Ok(FulfillArgs) if successful
	/// This should be called before the callback
	pub fn prepare_fulfill(&self) -> Result<FulfillArgs<T>, DispatchError> {
		ensure!(
			self.request.can_be_fulfilled(),
			Error::<T>::RequestCannotYetBeFulfilled
		);
		// get the randomness corresponding to the request
		let randomness: T::Hash = self.get_randomness()?;
		// No event emitted until fulfillment is complete
		Ok(FulfillArgs {
			request: self.request.clone(),
			deposit: self.deposit,
			contract_address: self.request.contract_address.clone(),
			gas_limit: self.request.fee,
			randomness,
		})
	}
	pub fn increase_fee(&mut self, caller: &T::AccountId, new_fee: BalanceOf<T>) -> DispatchResult {
		ensure!(
			caller == &self.request.contract_address,
			Error::<T>::OnlyRequesterCanIncreaseFee
		);
		let to_reserve = new_fee
			.checked_sub(&self.request.fee)
			.ok_or(Error::<T>::NewFeeMustBeGreaterThanOldFee)?;
		T::Currency::reserve(caller, to_reserve)?;
		self.request.fee = new_fee;
		Ok(())
	}
	/// Unreserve deposit + fee from contract_address
	/// Transfer fee to caller
	pub fn execute_expiration(&self, caller: &T::AccountId) -> DispatchResult {
		ensure!(
			frame_system::Pallet::<T>::block_number() >= self.expires,
			Error::<T>::RequestHasNotExpired
		);
		T::Currency::unreserve(
			&self.request.contract_address,
			self.deposit.saturating_add(self.request.fee),
		);
		T::Currency::transfer(
			&self.request.contract_address,
			caller,
			self.request.fee,
			KeepAlive,
		)
		.expect("just unreserved deposit + fee => fee must be transferrable");
		Ok(())
	}
}
