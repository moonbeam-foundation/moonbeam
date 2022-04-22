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

use crate::{traits::*, BalanceOf, Config, Error, Pallet};
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency};
use frame_support::weights::WeightToFeePolynomial;
use sp_runtime::traits::{CheckedSub, Saturating};

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Randomness storage item from BABE
pub enum BabeRandomness<Epoch, BlockNumber> {
	OneEpochAgo(Epoch),
	TwoEpochsAgo(Epoch),
	CurrentBlock(BlockNumber),
}

#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Type of request
pub enum RequestType<BlockNumber> {
	/// Babe one epoch ago
	BabeOneEpochAgo(u64),
	/// Babe two epochs ago
	BabeTwoEpochsAgo(u64),
	/// Babe per block
	BabeCurrentBlock(BlockNumber),
	/// Local per block VRF output
	Local(BlockNumber),
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
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
	pub info: RequestType<T::BlockNumber>,
}

impl<T: Config> Request<T> {
	pub fn can_be_fulfilled(&self) -> bool {
		todo!()
		// todo match on RequestType and use CurrentEpochIndex to check if can be fulfilled
		// self.when <= frame_system::Pallet::<T>::block_number()
	}
}

#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct RequestState<T: Config> {
	/// Fee is returned to this account upon execution
	pub request: Request<T>,
	/// Deposit taken for making request (stored in case config changes)
	pub deposit: BalanceOf<T>,
	/// All requests expire `T::ExpirationDelay` blocks after they are made
	pub expires: T::BlockNumber,
}

impl<T: Config> RequestState<T> {
	pub fn new(
		request: Request<T>,
		deposit: BalanceOf<T>,
	) -> Result<RequestState<T>, DispatchError> {
		let expires =
			frame_system::Pallet::<T>::block_number().saturating_add(T::ExpirationDelay::get());
		// ensure!(
		// 	request.info.when() < expires,
		// 	Error::<T>::InvalidRequestCannotBeFulfilledBeforeExpiry
		// );
		Ok(RequestState {
			request,
			deposit,
			expires,
		})
	}
	pub fn fulfill(&self, caller: &T::AccountId) -> DispatchResult {
		ensure!(
			self.request.can_be_fulfilled(),
			Error::<T>::RequestCannotYetBeFulfilled
		);
		let raw_randomness: T::Hash = match self.request.info {
			RequestType::Local { .. } => T::Hash::default(), // TODO
			_ => return Err(Error::<T>::NotYetImplemented.into()),
		};
		let randomness = Pallet::<T>::concat_and_hash(raw_randomness, self.request.salt);
		// add self.request.fee as input to send_randomness as gas_limit and if fails, do not delete request
		// convert it into gas
		// fulfillment benchmarking must exclude send_randomness callback
		// if send_randomness fails oog, then revert it and keep the request in storage
		// might need some special event when execution fails oog
		// responsibility of requester that callback succeeds => if it fails,
		// pass gas_limit and fees
		// check fees match gas_limit
		T::RandomnessSender::send_randomness(self.request.contract_address.clone(), randomness);
		// return deposit + fee_excess to contract_address
		// refund cost_of_execution to caller?
		T::Currency::unreserve(
			&self.request.contract_address,
			self.deposit + self.request.fee,
		);
		// get cost of execution from EVM
		let execution_weight_estimate: Weight = 0; // TODO accurate estimate of execution weight
		let execution_fee_estimate = T::WeightToFee::calc(&execution_weight_estimate);
		let refund_fee = self.request.fee.saturating_sub(execution_fee_estimate);
		// refund excess fee to refund address
		// TODO: withdraw like Elois's orbiters PR
		T::Currency::transfer(
			&self.request.contract_address,
			&self.request.refund_address,
			refund_fee,
			KeepAlive,
		)
		.expect("just unreserved deposit + fee => refund_fee must be transferrable");
		// refund caller
		// TODO: withdraw like Elois's orbiters PR
		T::Currency::transfer(
			&self.request.contract_address,
			caller,
			execution_fee_estimate,
			KeepAlive,
		)
		.expect("just unreserved deposit + fee => execution_fee_estimate must be transferrable");
		Ok(())
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
