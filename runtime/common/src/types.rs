use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Encode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(LOWER, UPPER))]
pub struct BoundedU128<const LOWER: u128, const UPPER: u128>(u128);

impl<const L: u128, const U: u128> BoundedU128<L, U> {
	pub fn new(value: u128) -> Result<Self, &'static str> {
		if value < L || value > U {
			return Err("Value out of bounds");
		}
		Ok(Self(value))
	}

	pub fn new_or_min(value: u128) -> Self {
		if value < L || value > U {
			Self(L)
		} else {
			Self(value)
		}
	}

	pub fn value(&self) -> u128 {
		self.0
	}
}

impl<const L: u128, const U: u128> Decode for BoundedU128<L, U> {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		let value = u128::decode(input)?;
		if value < L || value > U {
			return Err("Value out of bounds".into());
		}
		Ok(Self(value))
	}
}

impl<const L: u128, const U: u128> EncodeLike<u128> for BoundedU128<L, U> {}

#[macro_export]
macro_rules! expose_u128_get {
	($name:ident,$bounded_get:ty) => {
		pub struct $name;

		impl sp_core::Get<u128> for $name {
			fn get() -> u128 {
				<$bounded_get>::get().value()
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;
	use sp_core::Get;

	use super::*;

	#[test]
	fn test_bounded_u128() {
		let bounded = BoundedU128::<1, 10>::new(5).unwrap();
		assert_eq!(bounded.value(), 5);

		let bounded = BoundedU128::<1, 10>::new(0);
		assert_eq!(bounded, Err("Value out of bounds"));

		let bounded = BoundedU128::<1, 10>::new(11);
		assert_eq!(bounded, Err("Value out of bounds"));

		let bounded = BoundedU128::<1, 10>::new_or_min(0);
		assert_eq!(bounded.value(), 1);

		let bounded = BoundedU128::<1, 10>::new_or_min(5);
		assert_eq!(bounded.value(), 5);

		let bounded = BoundedU128::<1, 10>::new_or_min(11);
		assert_eq!(bounded.value(), 1);
	}

	#[test]
	fn test_expose_u128_get() {
		parameter_types! {
			pub Bounded: BoundedU128::<1, 10> = BoundedU128::<1, 10>::new(4).unwrap();
		}
		expose_u128_get!(Exposed, Bounded);
		assert_eq!(Bounded::get().value(), Exposed::get());
	}

	#[test]
	fn test_encode_decode() {
		let bounded = BoundedU128::<1, 10>::new(5).unwrap();
		let encoded = bounded.encode();
		let decoded = BoundedU128::<1, 10>::decode(&mut &encoded[..]).unwrap();
		assert_eq!(bounded, decoded);
	}

	#[test]
	fn test_encode_invalid() {
		let bounded = BoundedU128::<1, 10>::new(9);
		let encoded = bounded.encode();
		let decoded = BoundedU128::<1, 3>::decode(&mut &encoded[..]);
		assert_eq!(decoded, Err("Value out of bounds".into()));

		let bounded = BoundedU128::<1, 10>::new(9);
		let encoded = bounded.encode();
		let decoded = BoundedU128::<100, 500>::decode(&mut &encoded[..]);
		assert_eq!(decoded, Err("Value out of bounds".into()));
	}
}
