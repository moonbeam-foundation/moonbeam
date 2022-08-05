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

#[derive(PartialEq, Eq)]
enum ErrorLocationPart {
	Field(String),
	Array(usize),
}

/// Location of an error.
/// Built depth-first.
#[derive(PartialEq, Eq)]
pub struct ErrorLocation(Vec<ErrorLocationPart>);

impl ErrorLocation {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl core::fmt::Display for ErrorLocation {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		for (i, part) in self.0.iter().rev().enumerate() {
			match (i, part) {
				(0, ErrorLocationPart::Field(field)) => write!(f, "{field}")?,
				(_, ErrorLocationPart::Field(field)) => write!(f, ".{field}")?,
				(_, ErrorLocationPart::Array(index)) => write!(f, "[{index}]")?,
			}
		}
		Ok(())
	}
}

/// Kind of error.
#[non_exhaustive]
#[derive(PartialEq, Eq)]
pub enum ErrorKind {
	Custom(String),
	ReadOutOfBounds { what: String },
	UnknownSelector,
	InputIsTooShort,
	ValueIsTooLarge { what: String },
	PointerToOutofBound,
	CursorOverflow,
	ExpectedAtLeastNArguments(usize),
}

impl ErrorKind {
	pub fn custom(s: impl Into<String>) -> Self {
		ErrorKind::Custom(s.into())
	}

	pub fn read_out_of_bounds(what: impl Into<String>) -> Self {
		ErrorKind::ReadOutOfBounds { what: what.into() }
	}

	pub fn value_is_too_large(what: impl Into<String>) -> Self {
		ErrorKind::ValueIsTooLarge { what: what.into() }
	}
}

impl core::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		match self {
			ErrorKind::Custom(s) => write!(f, "{s}"),
			ErrorKind::ReadOutOfBounds { what } => write!(f, "Tried to read {what} out of bounds"),
			ErrorKind::UnknownSelector => write!(f, "Unknown selector"),
			ErrorKind::InputIsTooShort => write!(f, "InputIsTooShort"),
			ErrorKind::ValueIsTooLarge { what } => write!(f, "Value is too large for {what}"),
			ErrorKind::PointerToOutofBound => write!(f, "Pointer points to out of bound"),
			ErrorKind::CursorOverflow => write!(f, "Reading cursor overflowed"),
			ErrorKind::ExpectedAtLeastNArguments(n) => write!(f, "Expected at least {n} arguments"),
		}
	}
}

/// An error returned by various functions in precompile-utils.
/// Allows to dynamically construct the location (backtrace) of the error
/// and manage the error in a typed way.
/// Can be transformed into a `PrecompileFailure::Revert` and `String`, and
/// implement `Display` and `Debug`.
#[derive(PartialEq, Eq)]
pub struct Error {
	kind: ErrorKind,
	location: ErrorLocation,
}

impl Error {
	pub fn new(kind: ErrorKind) -> Self {
		Self {
			kind,
			location: ErrorLocation::new(),
		}
	}

	pub fn change_what(mut self, what: impl Into<String>) -> Self {
		let what = what.into();

		self.kind = match self.kind {
			ErrorKind::ReadOutOfBounds { .. } => ErrorKind::ReadOutOfBounds { what },
			ErrorKind::ValueIsTooLarge { .. } => ErrorKind::ValueIsTooLarge { what },
			other => other,
		};

		self
	}

	pub fn to_bytes(self) -> Vec<u8> {
		self.into()
	}
}

impl From<ErrorKind> for Error {
	fn from(a: ErrorKind) -> Error {
		Error::new(a)
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		if !self.location.is_empty() {
			write!(f, "{}: ", self.location)?;
		}

		write!(f, "{}", self.kind)
	}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self)
	}
}

impl Into<Vec<u8>> for Error {
	fn into(self) -> Vec<u8> {
		self.to_string().into()
	}
}

pub trait LocationMap {
	fn in_field(self, field: impl Into<String>) -> Self;
	fn in_array(self, index: usize) -> Self;
}

impl LocationMap for ErrorLocation {
	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.0.push(ErrorLocationPart::Field(field.into()));
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.0.push(ErrorLocationPart::Array(index));
		self
	}
}

impl LocationMap for Error {
	fn in_field(mut self, field: impl Into<String>) -> Self {
		self.location = self.location.in_field(field);
		self
	}

	fn in_array(mut self, index: usize) -> Self {
		self.location = self.location.in_array(index);
		self
	}
}

impl<T> LocationMap for Result<T, Error> {
	fn in_field(self, field: impl Into<String>) -> Self {
		self.map_err(|e| e.in_field(field))
	}

	fn in_array(self, index: usize) -> Self {
		self.map_err(|e| e.in_array(index))
	}
}

impl From<Error> for crate::PrecompileFailure {
	fn from(err: Error) -> Self {
		crate::revert(err.to_string())
	}
}
