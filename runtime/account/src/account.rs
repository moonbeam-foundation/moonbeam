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


use sp_core::{H160};
use sp_std::hash::Hash;
use sp_std::vec::Vec;
use sp_std::str;
use sp_std::convert::TryFrom;
use codec::{Encode, Decode};
#[cfg(feature = "std")]
use sp_core::hexdisplay::HexDisplay;
use zeroize::Zeroize;
#[doc(hidden)]
pub use sp_std::ops::Deref;
use sp_runtime_interface::pass_by::PassByInner;
#[cfg(feature = "std")]
use std::str::FromStr;

use pallet_evm::AddressMapping;

/// The root phrase for our publicly known keys.
pub const DEV_PHRASE: &str = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";

/// The address of the associated root phrase for our publicly known keys.
pub const DEV_ADDRESS: &str = "5DfhGyQdFobKM8NsWvEeAKk5EQQgYe9AydgJ7rMB6E1EqRzV";

/// The infallible type.
#[derive(sp_core::RuntimeDebug)]
pub enum Infallible {}

/// The length of the junction identifier. Note that this is also referred to as the
/// `CHAIN_CODE_LENGTH` in the context of Schnorrkel.
#[cfg(feature = "full_crypto")]
pub const JUNCTION_ID_LEN: usize = 20;

/// Similar to `From`, except that the onus is on the part of the caller to ensure
/// that data passed in makes sense. Basically, you're not guaranteed to get anything
/// sensible out.
pub trait UncheckedFrom<T> {
	/// Convert from an instance of `T` to Self. This is not guaranteed to be
	/// whatever counts as a valid instance of `T` and it's up to the caller to
	/// ensure that it makes sense.
	fn unchecked_from(t: T) -> Self;
}

/// The counterpart to `UncheckedFrom`.
pub trait UncheckedInto<T> {
	/// The counterpart to `unchecked_from`.
	fn unchecked_into(self) -> T;
}

impl<S, T: UncheckedFrom<S>> UncheckedInto<T> for S {
	fn unchecked_into(self) -> T {
		T::unchecked_from(self)
	}
}

/// A store for sensitive data.
///
/// Calls `Zeroize::zeroize` upon `Drop`.
#[derive(Clone)]
pub struct Protected<T: Zeroize>(T);

impl<T: Zeroize> AsRef<T> for Protected<T> {
	fn as_ref(&self) -> &T {
		&self.0
	}
}

impl<T: Zeroize> sp_std::ops::Deref for Protected<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.0
	}
}

#[cfg(feature = "std")]
impl<T: Zeroize> std::fmt::Debug for Protected<T> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "<protected>")
	}
}

impl<T: Zeroize> From<T> for Protected<T> {
	fn from(t: T) -> Self {
		Protected(t)
	}
}

impl<T: Zeroize> Zeroize for Protected<T> {
	fn zeroize(&mut self) {
		self.0.zeroize()
	}
}

impl<T: Zeroize> Drop for Protected<T> {
	fn drop(&mut self) {
		self.zeroize()
	}
}

/// An error with the interpretation of a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "full_crypto")]
pub enum SecretStringError {
	/// The overall format was invalid (e.g. the seed phrase contained symbols).
	InvalidFormat,
	/// The seed phrase provided is not a valid BIP39 phrase.
	InvalidPhrase,
	/// The supplied password was invalid.
	InvalidPassword,
	/// The seed is invalid (bad content).
	InvalidSeed,
	/// The seed has an invalid length.
	InvalidSeedLength,
	/// The derivation path was invalid (e.g. contains soft junctions when they are not supported).
	InvalidPath,
}

/// A since derivation junction description. It is the single parameter used when creating
/// a new secret key from an existing secret key and, in the case of `SoftRaw` and `SoftIndex`
/// a new public key from an existing public key.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Encode, Decode)]
#[cfg(feature = "full_crypto")]
pub enum DeriveJunction {
	/// Soft (vanilla) derivation. Public keys have a correspondent derivation.
	Soft([u8; JUNCTION_ID_LEN]),
	/// Hard ("hardened") derivation. Public keys do not have a correspondent derivation.
	Hard([u8; JUNCTION_ID_LEN]),
}

#[cfg(feature = "full_crypto")]
impl DeriveJunction {
	/// Consume self to return a soft derive junction with the same chain code.
	pub fn soften(self) -> Self { DeriveJunction::Soft(self.unwrap_inner()) }

	/// Consume self to return a hard derive junction with the same chain code.
	pub fn harden(self) -> Self { DeriveJunction::Hard(self.unwrap_inner()) }

	/// Create a new soft (vanilla) DeriveJunction from a given, encodable, value.
	///
	/// If you need a hard junction, use `hard()`.
	pub fn soft<T: Encode>(index: T) -> Self {
		let mut cc: [u8; JUNCTION_ID_LEN] = Default::default();
		index.using_encoded(|data| if data.len() > JUNCTION_ID_LEN {
			let hash_result = blake2_rfc::blake2b::blake2b(JUNCTION_ID_LEN, &[], data);
			let hash = hash_result.as_bytes();
			cc.copy_from_slice(hash);
		} else {
			cc[0..data.len()].copy_from_slice(data);
		});
		DeriveJunction::Soft(cc)
	}

	/// Create a new hard (hardened) DeriveJunction from a given, encodable, value.
	///
	/// If you need a soft junction, use `soft()`.
	pub fn hard<T: Encode>(index: T) -> Self {
		Self::soft(index).harden()
	}

	/// Consume self to return the chain code.
	pub fn unwrap_inner(self) -> [u8; JUNCTION_ID_LEN] {
		match self {
			DeriveJunction::Hard(c) | DeriveJunction::Soft(c) => c,
		}
	}

	/// Get a reference to the inner junction id.
	pub fn inner(&self) -> &[u8; JUNCTION_ID_LEN] {
		match self {
			DeriveJunction::Hard(ref c) | DeriveJunction::Soft(ref c) => c,
		}
	}

	/// Return `true` if the junction is soft.
	pub fn is_soft(&self) -> bool {
		match *self {
			DeriveJunction::Soft(_) => true,
			_ => false,
		}
	}

	/// Return `true` if the junction is hard.
	pub fn is_hard(&self) -> bool {
		match *self {
			DeriveJunction::Hard(_) => true,
			_ => false,
		}
	}
}

#[cfg(feature = "full_crypto")]
impl<T: AsRef<str>> From<T> for DeriveJunction {
	fn from(j: T) -> DeriveJunction {
		let j = j.as_ref();
		let (code, hard) = if j.starts_with("/") {
			(&j[1..], true)
		} else {
			(j, false)
		};

		let res = if let Ok(n) = str::parse::<u64>(code) {
			// number
			DeriveJunction::soft(n)
		} else {
			// something else
			DeriveJunction::soft(code)
		};

		if hard {
			res.harden()
		} else {
			res
		}
	}
}

/// Derivable key trait.
pub trait Derive: Sized {
	/// Derive a child key from a series of given junctions.
	///
	/// Will be `None` for public keys if there are any hard junctions in there.
	#[cfg(feature = "std")]
	fn derive<Iter: Iterator<Item=DeriveJunction>>(&self, _path: Iter) -> Option<Self> {
		None
	}
}

/// Trait suitable for typical cryptographic PKI key public type.
pub trait Public:
	AsRef<[u8]> + AsMut<[u8]> + Default + Derive + PartialEq + Eq + Clone + Send + Sync
{
	/// A new instance from the given slice.
	///
	/// NOTE: No checking goes on to ensure this is a real public key. Only use it if
	/// you are certain that the array actually is a pubkey. GIGO!
	fn from_slice(data: &[u8]) -> Self;

	/// Return a `Vec<u8>` filled with raw data.
	fn to_raw_vec(&self) -> Vec<u8> { self.as_slice().to_vec() }

	/// Return a slice filled with raw data.
	fn as_slice(&self) -> &[u8] { self.as_ref() }
	/// Return `CryptoTypePublicPair` from public key.
	fn to_public_crypto_pair(&self) -> CryptoTypePublicPair;
}


/// An error type for SS58 decoding.
#[cfg(feature = "full_crypto")]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum PublicError {
	/// Bad length.
	BadLength,
	/// Invalid format.
	InvalidFormat,
	/// Invalid derivation path.
	InvalidPath,
}

/// An opaque 20-byte cryptographic identifier.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Default, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct AccountId20([u8; 20]);

#[cfg(feature = "std")]
impl sp_std::str::FromStr for AccountId20 {
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


impl UncheckedFrom<sp_core::hash::H160> for AccountId20 {
	fn unchecked_from(h: sp_core::hash::H160) -> Self {
		AccountId20(h.into())
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
		write!(f, "{}", sp_core::hexdisplay::HexDisplay::from(&self.0))
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

/// One type is wrapped by another.
pub trait IsWrappedBy<Outer>: From<Outer> + Into<Outer> {
	/// Get a reference to the inner from the outer.
	fn from_ref(outer: &Outer) -> &Self;
	/// Get a mutable reference to the inner from the outer.
	fn from_mut(outer: &mut Outer) -> &mut Self;
}

/// Opposite of `IsWrappedBy` - denotes a type which is a simple wrapper around another type.
pub trait Wraps: Sized {
	/// The inner type it is wrapping.
	type Inner: IsWrappedBy<Self>;
}

impl<T, Outer> IsWrappedBy<Outer> for T where
	Outer: AsRef<Self> + AsMut<Self> + From<Self>,
	T: From<Outer>,
{
	/// Get a reference to the inner from the outer.
	fn from_ref(outer: &Outer) -> &Self { outer.as_ref() }

	/// Get a mutable reference to the inner from the outer.
	fn from_mut(outer: &mut Outer) -> &mut Self { outer.as_mut() }
}

impl<Inner, Outer, T> UncheckedFrom<T> for Outer where
	Outer: Wraps<Inner=Inner>,
	Inner: IsWrappedBy<Outer> + UncheckedFrom<T>,
{
	fn unchecked_from(t: T) -> Self {
		let inner: Inner = t.unchecked_into();
		inner.into()
	}
}

/// An identifier for a type of cryptographic key.
///
/// To avoid clashes with other modules when distributing your module publicly, register your
/// `KeyTypeId` on the list here by making a PR.
///
/// Values whose first character is `_` are reserved for private use and won't conflict with any
/// public modules.
#[derive(
	Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, PassByInner,
	sp_core::RuntimeDebug
)]
pub struct KeyTypeId(pub [u8; 4]);

impl From<u32> for KeyTypeId {
	fn from(x: u32) -> Self {
		Self(x.to_le_bytes())
	}
}

impl From<KeyTypeId> for u32 {
	fn from(x: KeyTypeId) -> Self {
		u32::from_le_bytes(x.0)
	}
}

impl<'a> TryFrom<&'a str> for KeyTypeId {
	type Error = ();
	fn try_from(x: &'a str) -> Result<Self, ()> {
		let b = x.as_bytes();
		if b.len() != 4 {
			return Err(());
		}
		let mut res = KeyTypeId::default();
		res.0.copy_from_slice(&b[0..4]);
		Ok(res)
	}
}

/// An identifier for a specific cryptographic algorithm used by a key pair
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CryptoTypeId(pub [u8; 4]);

/// A type alias of CryptoTypeId & a public key
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CryptoTypePublicPair(pub CryptoTypeId, pub Vec<u8>);

#[cfg(feature = "std")]
impl sp_std::fmt::Display for CryptoTypePublicPair {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		let id = match str::from_utf8(&(self.0).0[..]) {
			Ok(id) => id.to_string(),
			Err(_) => {
				format!("{:#?}", self.0)
			}
		};
		write!(f, "{}-{}", id, HexDisplay::from(&self.1))
	}
}
