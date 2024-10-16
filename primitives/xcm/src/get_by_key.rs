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

/// A trait for querying a value by a key.
pub trait GetByKey<Key, Value> {
	/// Return the value.
	fn get(k: &Key) -> Value;
}

/// Create new implementations of the `GetByKey` trait.
///
/// The implementation is typically used like a map or set.
///
/// Example:
/// ```ignore
/// use primitives::CurrencyId;
/// parameter_type_with_key! {
///     pub Rates: |currency_id: CurrencyId| -> u32 {
///         match currency_id {
///             CurrencyId::DOT => 1,
///             CurrencyId::KSM => 2,
///             _ => 3,
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! parameter_type_with_key {
	(
		pub $name:ident: |$k:ident: $key:ty| -> $value:ty $body:block;
	) => {
		pub struct $name;
		impl $crate::get_by_key::GetByKey<$key, $value> for $name {
			fn get($k: &$key) -> $value {
				$body
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;

	parameter_type_with_key! {
		pub Test: |k: u32| -> u32 {
			match k {
				1 => 1,
				_ => 2,
			}
		};
	}

	#[test]
	fn get_by_key_should_work() {
		assert_eq!(Test::get(&1), 1);
		assert_eq!(Test::get(&2), 2);
		assert_eq!(Test::get(&3), 2);
	}
}
