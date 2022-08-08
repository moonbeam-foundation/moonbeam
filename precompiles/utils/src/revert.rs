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

use crate::{Bytes, EvmDataWriter};
use alloc::string::{String, ToString};
use fp_evm::{ExitRevert, PrecompileFailure};
use sp_std::vec::Vec;

/// Represent the result of a computation that can revert.
pub type MayRevert<T = ()> = Result<T, Revert>;

/// Generic error to build abi-encoded revert output.
/// See: https://docs.soliditylang.org/en/latest/control-structures.html?highlight=revert#revert
#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum RevertSelector {
	Generic = "Error(string)",
}

#[derive(PartialEq, Eq)]
enum BacktracePart {
	Field(String),
	Array(usize),
}

/// Location of an error.
/// Built depth-first.
#[derive(PartialEq, Eq)]
pub struct Backtrace(Vec<BacktracePart>);

impl Backtrace {
	pub fn new() -> Self {
		Self(Vec::new())
	}

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
				(_, BacktracePart::Array(index)) => write!(f, "[{index}]")?,
			}
		}
		Ok(())
	}
}

/// Kind of error.
#[non_exhaustive]
#[derive(PartialEq, Eq)]
pub enum RevertReason {
	Custom(String),
	ReadOutOfBounds { what: String },
	UnknownSelector,
	InputIsTooShort,
	ValueIsTooLarge { what: String },
	PointerToOutofBound,
	CursorOverflow,
	ExpectedAtLeastNArguments(usize),
}

impl RevertReason {
	pub fn custom(s: impl Into<String>) -> Self {
		RevertReason::Custom(s.into())
	}

	pub fn read_out_of_bounds(what: impl Into<String>) -> Self {
		RevertReason::ReadOutOfBounds { what: what.into() }
	}

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
			RevertReason::InputIsTooShort => write!(f, "InputIsTooShort"),
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
/// Allows to dynamically construct the location (backtrace) of the revert
/// and manage it in a typed way.
/// Can be transformed into a `PrecompileFailure::Revert` and `String`, and
/// implement `Display` and `Debug`.
#[derive(PartialEq, Eq)]
pub struct Revert {
	kind: RevertReason,
	location: Backtrace,
}

impl Revert {
	pub fn new(kind: RevertReason) -> Self {
		Self {
			kind,
			location: Backtrace::new(),
		}
	}

	pub fn change_what(mut self, what: impl Into<String>) -> Self {
		let what = what.into();

		self.kind = match self.kind {
			RevertReason::ReadOutOfBounds { .. } => RevertReason::ReadOutOfBounds { what },
			RevertReason::ValueIsTooLarge { .. } => RevertReason::ValueIsTooLarge { what },
			other => other,
		};

		self
	}

	pub fn to_bytes(self) -> Vec<u8> {
		self.into()
	}
}

impl From<RevertReason> for Revert {
	fn from(a: RevertReason) -> Revert {
		Revert::new(a)
	}
}

impl core::fmt::Display for Revert {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
		if !self.location.is_empty() {
			write!(f, "{}: ", self.location)?;
		}

		write!(f, "{}", self.kind)
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

pub trait BacktraceExt {
	fn in_field(self, field: impl Into<String>) -> Self;
	fn in_array(self, index: usize) -> Self;
}

impl BacktraceExt for Backtrace {
	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.0.push(BacktracePart::Field(field.into()));
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.0.push(BacktracePart::Array(index));
		self
	}
}

impl BacktraceExt for Revert {
	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.location = self.location.in_field(field);
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.location = self.location.in_array(index);
		self
	}
}

impl<T> BacktraceExt for Result<T, Revert> {
	fn in_field(self, field: impl Into<String>) -> Self {
		self.map_err(|e| e.in_field(field))
	}

	fn in_array(self, index: usize) -> Self {
		self.map_err(|e| e.in_array(index))
	}
}

impl From<Revert> for PrecompileFailure {
	fn from(err: Revert) -> Self {
		PrecompileFailure::Revert {
			exit_status: ExitRevert::Reverted,
			output: EvmDataWriter::new_with_selector(RevertSelector::Generic)
				.write::<Bytes>(Bytes(err.to_string().into()))
				.build(),
		}
	}
}

impl From<RevertReason> for PrecompileFailure {
	fn from(err: RevertReason) -> Self {
		Revert::new(err).into()
	}
}
