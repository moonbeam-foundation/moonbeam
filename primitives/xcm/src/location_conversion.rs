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

use parity_scale_codec::Encode;
use sp_io::hashing::blake2_256;
use sp_std::borrow::Borrow;
use sp_std::marker::PhantomData;
use xcm::latest::MultiLocation;

use xcm_executor::traits::Convert;

// Struct that converts a given MultiLocation into a 20 bytes account id by hashing
// with blake2_256 and taking the first 20 bytes
pub struct Account20Hash<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 20]> + Into<[u8; 20]> + Clone> Convert<MultiLocation, AccountId>
	for Account20Hash<AccountId>
{
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		let hash: [u8; 32] = ("multiloc", location.borrow())
			.borrow()
			.using_encoded(blake2_256);
		let mut account_id = [0u8; 20];
		account_id.copy_from_slice(&hash[0..20]);
		Ok(account_id.into())
	}

	fn reverse_ref(_: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		Err(())
	}
}
