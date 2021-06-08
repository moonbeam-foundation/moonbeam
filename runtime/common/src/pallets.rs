// Copyright 2019-2021 PureStake Inc.
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

#[macro_export]
macro_rules! impl_pallet_evm {
	() => {
		/// Current approximation of the gas/s consumption considering
		/// EVM execution over compiled WASM (on 4.4Ghz CPU).
		/// Given the 500ms Weight, from which 75% only are used for transactions,
		/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
		pub const GAS_PER_SECOND: u64 = 40_000_000;

		/// Approximate ratio of the amount of Weight per Gas.
		/// u64 works for approximations because Weight is a very small unit compared to gas.
		pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

		pub struct MoonbeamGasWeightMapping;

		impl pallet_evm::GasWeightMapping for MoonbeamGasWeightMapping {
			fn gas_to_weight(gas: u64) -> Weight {
				gas.saturating_mul(WEIGHT_PER_GAS)
			}
			fn weight_to_gas(weight: Weight) -> u64 {
				u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
			}
		}

		parameter_types! {
			pub BlockGasLimit: U256
				= U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
		}

		pub struct FixedGasPrice;
		impl FeeCalculator for FixedGasPrice {
			fn min_gas_price() -> U256 {
				1_000_000_000.into()
			}
		}

		impl pallet_evm::Config for Runtime {
			type FeeCalculator = FixedGasPrice;
			type GasWeightMapping = MoonbeamGasWeightMapping;
			type CallOrigin = EnsureAddressRoot<AccountId>;
			type WithdrawOrigin = EnsureAddressNever<AccountId>;
			type AddressMapping = IdentityAddressMapping;
			type Currency = Balances;
			type Event = Event;
			type Runner = pallet_evm::runner::stack::Runner<Self>;
			type Precompiles = MoonbeamPrecompiles<Self>;
			type ChainId = EthereumChainId;
			type OnChargeTransaction = ();
			type BlockGasLimit = BlockGasLimit;
		}
	};
}
