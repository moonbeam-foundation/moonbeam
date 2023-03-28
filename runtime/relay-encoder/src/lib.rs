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

//! Encoder for relay runtimes
#![cfg_attr(not(feature = "std"), no_std)]

pub mod kusama;
pub mod polkadot;
pub mod westend;

// #[cfg(test)]
// mod common_encoder_tests;

use pallet_xcm_transactor::relay_indices::*;

/// Westend hardcoded indices
pub const WESTEND_RELAY_INDICES: RelayChainIndices = RelayChainIndices {
	pallets: PalletIndices {
		staking: 6u8,
		utility: 16u8,
		hrmp: 51u8,
	},
	calls: CallIndices {
		staking: StakingIndices {
			bond: 0u8,
			bond_extra: 1u8,
			unbond: 2u8,
			withdraw_unbonded: 3u8,
			validate: 4u8,
			nominate: 5u8,
			chill: 6u8,
			set_payee: 7u8,
			set_controller: 8u8,
			rebond: 19u8,
		},
		utility: UtilityIndices { as_derivative: 1u8 },
		hrmp: HrmpIndices {
			init_open_channel: 0u8,
			accept_open_channel: 1u8,
			close_channel: 2u8,
		},
	},
};

/// Kusama hardcoded indices
pub const KUSAMA_RELAY_INDICES: RelayChainIndices = RelayChainIndices {
	pallets: PalletIndices {
		staking: 6u8,
		utility: 24u8,
		hrmp: 1u8,
	},
	calls: CallIndices {
		staking: StakingIndices {
			bond: 1u8,
			bond_extra: 1u8,
			unbond: 1u8,
			withdraw_unbonded: 1u8,
			validate: 1u8,
			nominate: 1u8,
			chill: 6u8,
			set_payee: 1u8,
			set_controller: 1u8,
			rebond: 1u8,
		},
		utility: UtilityIndices { as_derivative: 1u8 },
		hrmp: HrmpIndices {
			init_open_channel: 1u8,
			accept_open_channel: 1u8,
			close_channel: 1u8,
		},
	},
};

/// Polkadot hardcoded relay indices
pub const POLKADOT_RELAY_INDICES: RelayChainIndices = RelayChainIndices {
	pallets: PalletIndices {
		staking: 6u8,
		utility: 24u8,
		hrmp: 1u8,
	},
	calls: CallIndices {
		staking: StakingIndices {
			bond: 1u8,
			bond_extra: 1u8,
			unbond: 1u8,
			withdraw_unbonded: 1u8,
			validate: 1u8,
			nominate: 1u8,
			chill: 6u8,
			set_payee: 1u8,
			set_controller: 1u8,
			rebond: 1u8,
		},
		utility: UtilityIndices { as_derivative: 1u8 },
		hrmp: HrmpIndices {
			init_open_channel: 1u8,
			accept_open_channel: 1u8,
			close_channel: 1u8,
		},
	},
};
