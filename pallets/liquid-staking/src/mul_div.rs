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

use {sp_core::U256, sp_runtime::traits::Zero, sp_std::convert::TryInto};

pub trait MulDiv: Sized {
	/// Multiply self by `a` then divide the result by `b`.
	/// Computation will be performed in a bigger type to avoid overflows.
	/// After the division, will return `None` if the result is to big for
	/// the real type or if `b` is zero.
	fn mul_div(self, a: Self, b: Self) -> Option<Self>;
}

macro_rules! impl_mul_div {
	($type:ty, $bigger:ty) => {
		impl MulDiv for $type {
			fn mul_div(self, a: Self, b: Self) -> Option<Self> {
				if self.is_zero() {
					return Some(<$type>::zero());
				}

				if b.is_zero() {
					return None;
				}

				let s: $bigger = self.into();
				let a: $bigger = a.into();
				let b: $bigger = b.into();

				let r: $bigger = s * a / b;

				r.try_into().ok()
			}
		}
	};
}

impl_mul_div!(u8, u16);
impl_mul_div!(u16, u32);
impl_mul_div!(u32, u64);
impl_mul_div!(u64, u128);
impl_mul_div!(u128, U256);
