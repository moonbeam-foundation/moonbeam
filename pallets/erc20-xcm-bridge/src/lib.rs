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

//! Minimal Pallet that stores the numeric Ethereum-style chain id in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use ethereum_types::BigEndianHash;
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
	use pallet_evm::Runner;
	use sp_core::{H160, H256, U256};
	use xcm::latest::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult};
	use xcm_executor::traits::{Convert, Error as MatchError, MatchesFungibles};
	use xcm_executor::Assets;

	const ERC20_TRANSFER_GAS_LIMIT: u64 = 200_000;
	const ERC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type AccountIdConverter: Convert<MultiLocation, H160>;
		type EvmRunner: Runner<Self>;
		type Erc20SovereignAccount: Get<H160>;
		type Matcher: MatchesFungibles<H160, U256>;
	}

	impl<T: Config> Pallet<T> {
		fn erc20_transfer(
			erc20_contract_address: H160,
			from: H160,
			to: H160,
			amount: U256,
		) -> XcmResult {
			// ERC20.transfer method hash
			let mut input = ERC20_TRANSFER_SELECTOR.to_vec();
			// append receiver address
			input.extend_from_slice(H256::from(to).as_bytes());
			// append amount to be transferred
			input.extend_from_slice(H256::from_uint(&amount).as_bytes());

			let exec_info = T::EvmRunner::call(
				from,
				erc20_contract_address,
				input,
				U256::default(),
				ERC20_TRANSFER_GAS_LIMIT,
				None,
				None,
				None,
				Default::default(),
				false,
				false,
				&fp_evm::Config::london(),
			)
			.map_err(|_| XcmError::FailedToTransactAsset("Fail to call erc20 contract"))?;

			ensure!(
				matches!(
					exec_info.exit_reason,
					ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
				),
				XcmError::FailedToTransactAsset("Erc20 contract transfer fail")
			);

			// return value is true.
			let mut bytes = [0u8; 32];
			U256::from(1).to_big_endian(&mut bytes);

			// Check return value to make sure not calling on empty contracts.
			ensure!(
				!exec_info.value.is_empty() && exec_info.value == bytes,
				XcmError::FailedToTransactAsset("Erc20 contract return invalid value")
			);

			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Pallet<T> {
		fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
			let (contract_address, amount) = T::Matcher::matches_fungibles(what)?;
			let who = T::AccountIdConverter::convert_ref(who)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			Self::erc20_transfer(
				contract_address,
				T::Erc20SovereignAccount::get(),
				who,
				amount,
			)
		}

		fn withdraw_asset(what: &MultiAsset, who: &MultiLocation) -> Result<Assets, XcmError> {
			let (contract_address, amount) = T::Matcher::matches_fungibles(what)?;
			let who = T::AccountIdConverter::convert_ref(who)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			Self::erc20_transfer(
				contract_address,
				who,
				T::Erc20SovereignAccount::get(),
				amount,
			)?;

			Ok(what.clone().into())
		}
	}
}
