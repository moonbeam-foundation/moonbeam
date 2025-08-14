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

#![cfg_attr(not(feature = "std"), no_std)]

use account::EthereumSignature;
use sp_runtime::traits::BlakeTwo256;
pub use sp_runtime::OpaqueExtrinsic;
use sp_runtime::{
	generic,
	traits::{IdentifyAccount, Verify},
};

pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = EthereumSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// AssetId type
pub type AssetId = u128;
/// Balance of an account.
pub type Balance = u128;
/// An index to a block.
pub type BlockNumber = u32;
/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Index of a transaction in the chain.
pub type Index = u32;
/// The address format for describing accounts.
pub type Address = AccountId;
/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;
/// Digest item type.
pub type DigestItem = generic::DigestItem;

pub mod well_known_relay_keys {
	use hex_literal::hex;

	pub const TIMESTAMP_NOW: &[u8] =
		&hex!["f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"];
}
