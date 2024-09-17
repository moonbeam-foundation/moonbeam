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
