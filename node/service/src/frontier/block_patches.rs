// Copyright 2024 Moonbeam foundation
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

//! This file aggregates ethereum specific block patches.
//!
//! Patches:
//!
//! [Moonbase]
//!
//! Block: 0xdadbbbfd8a7f177c466117a6f5635eff51d698d12cec9f5e2b360909621beb43
//! Description:
//! 	Block 2285348 includes transactions from the previous block.
//!
//! 	The problem was caused by this change in frontier, which moved `Pending::<T>::kill()`
//! 	from `on_initialize` to `on_finalize`: https://github.com/polkadot-evm/frontier/pull/638
//!
//! 	The bug was included with runtime 1603 in block 2285347.
//!
//! Github issue: https://github.com/moonbeam-foundation/moonbeam/issues/3026
//!

use sp_core::H256;

struct BlockPatch {
	pub hash: H256,
	pub invalid_transaction: Vec<H256>,
}

pub const MOONBASE_BLOCK_PATCHES: Vec<BlockPatch> = vec![BlockPatch {
	hash: hex_literal::hex!("dadbbbfd8a7f177c466117a6f5635eff51d698d12cec9f5e2b360909621beb43")
		.into(),
	invalid_transaction: vec![
		hex_literal::hex!("006a6843eb35ad35a9ea9a99affa8d81f1ed500253c98cc9c080d84171a0afb3")
			.into(),
		hex_literal::hex!("64c102f664eb435206ad4fcb49b526722176bcf74801c79473c3b5b2c281a243")
			.into(),
		hex_literal::hex!("f546335453b6e35ce7e236ee873c96ba3a22602b3acc4f45f5d68b33a76d79ca")
			.into(),
		hex_literal::hex!("4ed713ccd474fc33d2022a802f064cc012e3e37cd22891d4a89c7ba3d776f2db")
			.into(),
		hex_literal::hex!("a5355f86844bb23fe666b10b509543fa377a9e324513eb221e0a2c926a64cae4")
			.into(),
		hex_literal::hex!("c14791a3a392018fc3438f39cac1d572e8baadd4ed350e0355d1ca874a169e6a")
			.into(),
	],
}];
