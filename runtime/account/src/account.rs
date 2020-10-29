// Copyright 2019-2020 PureStake Inc.
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

//! The AccountId20 implementation.
//!
//! It includes AccountId conversations
//!
//! Ex:
//! use std::str::FromStr;
//! AccountId20::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap()

use sp_core::{H160};
use sp_std::hash::Hash;
use codec::{Encode, Decode};
#[cfg(feature = "std")]
use sp_core::hexdisplay::HexDisplay;

#[cfg(feature = "std")]
use std::str::FromStr;

#[cfg(feature = "full_crypto")]
use sp_core::crypto::{PublicError};

use pallet_evm::AddressMapping;

/// An opaque 20-byte cryptographic identifier.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct AccountId20([u8; 20]);

#[cfg(feature = "std")]
impl FromStr for AccountId20 {
	type Err = PublicError;
	fn from_str(s: &str) -> Result<Self, PublicError> {
		let hex_without_prefix = s.trim_start_matches("0x");
		if hex_without_prefix.len() == 40 {
			let mut bytes = [0u8; 20];
			hex::decode_to_slice(hex_without_prefix, &mut bytes)
				.map_err(|_| PublicError::InvalidFormat)
				.map(|_| Self::from(bytes))
		} else {
			Err(PublicError::BadLength)
		}
	}
}

impl AsRef<[u8]> for AccountId20 {
	fn as_ref(&self) -> &[u8] {
		&self.0[..]
	}
}

impl AsMut<[u8]> for AccountId20 {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.0[..]
	}
}

impl AsRef<[u8; 20]> for AccountId20 {
	fn as_ref(&self) -> &[u8; 20] {
		&self.0
	}
}

impl AsMut<[u8; 20]> for AccountId20 {
	fn as_mut(&mut self) -> &mut [u8; 20] {
		&mut self.0
	}
}

impl From<[u8; 20]> for AccountId20 {
	fn from(x: [u8; 20]) -> AccountId20 {
		AccountId20(x)
	}
}

impl From<H160> for AccountId20 {
	fn from(x: H160) -> AccountId20 {
		AccountId20(x.0)
	}
}

impl<'a> sp_std::convert::TryFrom<&'a [u8]> for AccountId20 {
	type Error = ();
	fn try_from(x: &'a [u8]) -> Result<AccountId20, ()> {
		if x.len() == 20 {
			let mut r = AccountId20::default();
			r.0.copy_from_slice(x);
			Ok(r)
		} else {
			Err(())
		}
	}
}

impl From<AccountId20> for [u8; 20] {
	fn from(x: AccountId20) -> [u8; 20] {
		x.0
	}
}

/// Identity address mapping, required for EVM address mapping
pub struct IdentityAddressMapping;

impl AddressMapping<AccountId20> for IdentityAddressMapping {
	fn into_account_id(address: H160) -> AccountId20 {
		address.to_fixed_bytes().into()
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for AccountId20 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		H160::fmt(&H160::from_slice(&self.0), f)
	}
}

impl sp_std::fmt::Debug for AccountId20 {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "{}", HexDisplay::from(&self.0))
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(feature = "std")]
impl serde::Serialize for AccountId20 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
		H160::from_slice(&self.0).serialize(serializer)
	}
}

#[cfg(feature = "std")]
impl<'de> serde::Deserialize<'de> for AccountId20 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
		AccountId20::from_str(&String::deserialize(deserializer)?)
			.map_err(|e| serde::de::Error::custom(format!("{:?}", e)))
	}
}