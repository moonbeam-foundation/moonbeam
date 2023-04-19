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

//! Utilities to work with revert messages with support for backtraces and
//! consistent formatting.

use crate::solidity::{self, codec::bytes::UnboundedBytes};
use alloc::string::{String, ToString};
use fp_evm::{ExitRevert, PrecompileFailure};
use sp_std::vec::Vec;

/// Represent the result of a computation that can revert.
pub type MayRevert<T = ()> = Result<T, Revert>;

/// Generate an encoded revert from a simple String.
/// Returns a `PrecompileFailure` that fits in an `EvmResult::Err`.
pub fn revert(msg: impl Into<String>) -> PrecompileFailure {
	RevertReason::custom(msg).into()
}

/// Generate an encoded revert from a simple String.
/// Returns a `Vec<u8>` in case `PrecompileFailure` is too high level.
pub fn revert_as_bytes(msg: impl Into<String>) -> Vec<u8> {
	Revert::new(RevertReason::custom(msg)).to_encoded_bytes()
}

/// Generic error to build abi-encoded revert output.
/// See: https://docs.soliditylang.org/en/latest/control-structures.html?highlight=revert#revert
pub const ERROR_SELECTOR: u32 = 0x08c379a0;

#[derive(Clone, PartialEq, Eq)]
enum BacktracePart {
	Field(String),
	Tuple(usize),
	Array(usize),
}

/// Backtrace of an revert.
/// Built depth-first.
/// Implement `Display` to render the backtrace as a string.
#[derive(Default, PartialEq, Eq)]
pub struct Backtrace(Vec<BacktracePart>);

impl Backtrace {
	/// Create a new empty backtrace.
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Check if the backtrace is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl core::fmt::Display for Backtrace {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		for (i, part) in self.0.iter().rev().enumerate() {
			match (i, part) {
				(0, BacktracePart::Field(field)) => write!(f, "{field}")?,
				(_, BacktracePart::Field(field)) => write!(f, ".{field}")?,
				(_, BacktracePart::Tuple(index)) => write!(f, ".{index}")?,
				(_, BacktracePart::Array(index)) => write!(f, "[{index}]")?,
			}
		}
		Ok(())
	}
}

/// Possible revert reasons.
#[non_exhaustive]
#[derive(PartialEq, Eq)]
pub enum RevertReason {
	/// A custom revert reason if other variants are not appropriate.
	Custom(String),
	/// Tried to read data out of bounds.
	ReadOutOfBounds {
		/// What was being read?
		what: String,
	},
	/// An unknown selector has been provided.
	UnknownSelector,
	/// A value is too large to fit in the wanted type.
	/// For security reasons integers are always parsed as `uint256` then
	/// casted to the wanted type. If the value overflows this type then this
	/// revert is used.
	ValueIsTooLarge {
		/// What was being read?
		what: String,
	},
	/// A pointer (used for structs and arrays) points out of bounds.
	PointerToOutofBound,
	/// The reading cursor overflowed.
	/// This should realistically never happen as it would require an input
	/// of length larger than 2^64, which would cost too much to be included
	/// in a block.
	CursorOverflow,
	/// Used by a check that the input contains at least N static arguments.
	/// Often use to return early if the input is too short.
	ExpectedAtLeastNArguments(usize),
}

impl RevertReason {
	/// Create a `RevertReason::Custom` from anything that can be converted to a `String`.
	/// Argument is the custom revert message.
	pub fn custom(s: impl Into<String>) -> Self {
		RevertReason::Custom(s.into())
	}

	/// Create a `RevertReason::ReadOutOfBounds` from anything that can be converted to a `String`.
	/// Argument names what was expected to be read.
	pub fn read_out_of_bounds(what: impl Into<String>) -> Self {
		RevertReason::ReadOutOfBounds { what: what.into() }
	}

	/// Create a `RevertReason::ValueIsTooLarge` from anything that can be converted to a `String`.
	/// Argument names what was expected to be read.
	pub fn value_is_too_large(what: impl Into<String>) -> Self {
		RevertReason::ValueIsTooLarge { what: what.into() }
	}
}

impl core::fmt::Display for RevertReason {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		match self {
			RevertReason::Custom(s) => write!(f, "{s}"),
			RevertReason::ReadOutOfBounds { what } => {
				write!(f, "Tried to read {what} out of bounds")
			}
			RevertReason::UnknownSelector => write!(f, "Unknown selector"),
			RevertReason::ValueIsTooLarge { what } => write!(f, "Value is too large for {what}"),
			RevertReason::PointerToOutofBound => write!(f, "Pointer points to out of bound"),
			RevertReason::CursorOverflow => write!(f, "Reading cursor overflowed"),
			RevertReason::ExpectedAtLeastNArguments(n) => {
				write!(f, "Expected at least {n} arguments")
			}
		}
	}
}

/// An revert returned by various functions in precompile-utils.
/// Allows to dynamically construct the backtrace (backtrace) of the revert
/// and manage it in a typed way.
/// Can be transformed into a `PrecompileFailure::Revert` and `String`, and
/// implement `Display` and `Debug`.
#[derive(PartialEq, Eq)]
pub struct Revert {
	reason: RevertReason,
	backtrace: Backtrace,
}

impl Revert {
	/// Create a new `Revert` with a `RevertReason` and
	/// an empty backtrace.
	pub fn new(reason: RevertReason) -> Self {
		Self {
			reason,
			backtrace: Backtrace::new(),
		}
	}

	/// For all `RevertReason` variants that have a `what` field, change its value.
	/// Otherwise do nothing.
	/// It is useful when writing custom types `solidity::Codec` implementations using
	/// simpler types.
	pub fn change_what(mut self, what: impl Into<String>) -> Self {
		let what = what.into();

		self.reason = match self.reason {
			RevertReason::ReadOutOfBounds { .. } => RevertReason::ReadOutOfBounds { what },
			RevertReason::ValueIsTooLarge { .. } => RevertReason::ValueIsTooLarge { what },
			other => other,
		};

		self
	}

	/// Transforms the revert into its bytes representation (from a String).
	pub fn to_encoded_bytes(self) -> Vec<u8> {
		let bytes: Vec<u8> = self.into();
		solidity::encode_with_selector(ERROR_SELECTOR, UnboundedBytes::from(bytes))
	}
}

impl From<RevertReason> for Revert {
	fn from(a: RevertReason) -> Revert {
		Revert::new(a)
	}
}

impl core::fmt::Display for Revert {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		if !self.backtrace.is_empty() {
			write!(f, "{}: ", self.backtrace)?;
		}

		write!(f, "{}", self.reason)
	}
}

impl core::fmt::Debug for Revert {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		write!(f, "{}", self)
	}
}

impl Into<Vec<u8>> for Revert {
	fn into(self) -> Vec<u8> {
		self.to_string().into()
	}
}

/// Allows to inject backtrace data.
pub trait InjectBacktrace {
	/// Output type of the injection.
	/// Should be a type that can hold a backtrace.
	type Output;

	/// Occurs in a field.
	fn in_field(self, field: impl Into<String>) -> Self::Output;

	/// Occurs in a tuple.
	fn in_tuple(self, index: usize) -> Self::Output;

	/// Occurs in an array at provided index.
	fn in_array(self, index: usize) -> Self::Output;
}

/// Additional function for everything having a Backtrace.
pub trait BacktraceExt {
	/// Map last tuple entry into a field.
	/// Does nothing if last entry is not a tuple.
	/// As in Solidity structs are equivalent to tuples and are tricky to parse correctly,
	/// it allows to parse any struct as a tuple (with the correct implementation in this crate) and
	/// then map tuple indices to struct fields.
	fn map_in_tuple_to_field(self, fields: &[&'static str]) -> Self;
}

/// Additional functions for Revert and MayRevert.
pub trait RevertExt {
	/// Map the reason while keeping the same backtrace.
	fn map_reason(self, f: impl FnOnce(RevertReason) -> RevertReason) -> Self;
}

impl InjectBacktrace for RevertReason {
	// `RevertReason` cannot hold a backtrace, thus it wraps
	// it into a `Revert`.
	type Output = Revert;

	fn in_field(self, field: impl Into<String>) -> Revert {
		Revert::new(self).in_field(field)
	}

	fn in_array(self, index: usize) -> Revert {
		Revert::new(self).in_array(index)
	}

	fn in_tuple(self, index: usize) -> Revert {
		Revert::new(self).in_tuple(index)
	}
}

impl InjectBacktrace for Backtrace {
	type Output = Self;

	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.0.push(BacktracePart::Field(field.into()));
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.0.push(BacktracePart::Array(index));
		self
	}

	fn in_tuple(mut self, index: usize) -> Self {
		self.0.push(BacktracePart::Tuple(index));
		self
	}
}

impl BacktraceExt for Backtrace {
	fn map_in_tuple_to_field(mut self, fields: &[&'static str]) -> Self {
		if let Some(entry) = self.0.last_mut() {
			if let BacktracePart::Tuple(index) = *entry {
				if let Some(field) = fields.get(index) {
					*entry = BacktracePart::Field(field.to_string())
				}
			}
		}
		self
	}
}

impl InjectBacktrace for Revert {
	type Output = Self;

	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.backtrace = self.backtrace.in_field(field);
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.backtrace = self.backtrace.in_array(index);
		self
	}

	fn in_tuple(mut self, index: usize) -> Self {
		self.backtrace = self.backtrace.in_tuple(index);
		self
	}
}

impl RevertExt for Revert {
	fn map_reason(mut self, f: impl FnOnce(RevertReason) -> RevertReason) -> Self {
		self.reason = f(self.reason);
		self
	}
}

impl BacktraceExt for Revert {
	fn map_in_tuple_to_field(mut self, fields: &[&'static str]) -> Self {
		self.backtrace = self.backtrace.map_in_tuple_to_field(fields);
		self
	}
}

impl<T> InjectBacktrace for MayRevert<T> {
	type Output = Self;

	fn in_field(self, field: impl Into<String>) -> Self {
		self.map_err(|e| e.in_field(field))
	}

	fn in_array(self, index: usize) -> Self {
		self.map_err(|e| e.in_array(index))
	}

	fn in_tuple(self, index: usize) -> Self {
		self.map_err(|e| e.in_tuple(index))
	}
}

impl<T> RevertExt for MayRevert<T> {
	fn map_reason(self, f: impl FnOnce(RevertReason) -> RevertReason) -> Self {
		self.map_err(|e| e.map_reason(f))
	}
}

impl<T> BacktraceExt for MayRevert<T> {
	fn map_in_tuple_to_field(self, fields: &[&'static str]) -> Self {
		self.map_err(|e| e.map_in_tuple_to_field(fields))
	}
}

impl From<Revert> for PrecompileFailure {
	fn from(err: Revert) -> Self {
		PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: err.to_encoded_bytes(),
		}
	}
}

impl From<RevertReason> for PrecompileFailure {
	fn from(err: RevertReason) -> Self {
		Revert::new(err).into()
	}
}
