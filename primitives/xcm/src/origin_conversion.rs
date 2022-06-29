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

use frame_support::traits::{Get, OriginTrait};
use orml_traits::location::{RelativeReserveProvider, Reserve};
use sp_std::{convert::TryInto, marker::PhantomData};
use xcm::latest::{Junction::AccountKey20, Junctions::*, MultiAsset, MultiLocation, NetworkId};

/// Instructs how to convert a 20 byte accountId into a MultiLocation
pub struct AccountIdToMultiLocation<AccountId>(sp_std::marker::PhantomData<AccountId>);
impl<AccountId> sp_runtime::traits::Convert<AccountId, MultiLocation>
	for AccountIdToMultiLocation<AccountId>
where
	AccountId: Into<[u8; 20]>,
{
	fn convert(account: AccountId) -> MultiLocation {
		MultiLocation {
			parents: 0,
			interior: X1(AccountKey20 {
				network: NetworkId::Any,
				key: account.into(),
			}),
		}
	}
}

// Convert a local Origin (i.e., a signed 20 byte account Origin)  to a Multilocation
pub struct SignedToAccountId20<Origin, AccountId, Network>(
	sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 20]>, Network: Get<NetworkId>>
	xcm_executor::traits::Convert<Origin, MultiLocation>
	for SignedToAccountId20<Origin, AccountId, Network>
where
	Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(frame_system::RawOrigin::Signed(who)) => Ok(AccountKey20 {
				key: who.into(),
				network: Network::get(),
			}
			.into()),
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

/// This struct offers uses RelativeReserveProvider to output relative views of multilocations
/// However, additionally accepts a MultiLocation that aims at representing the chain part
/// (parent: 1, Parachain(paraId)) of the absolute representation of our chain.
/// If a token reserve matches against this absolute view, we return  Some(MultiLocation::here())
/// This helps users by preventing errors when they try to transfer a token through xtokens
/// to our chain (either inserting the relative or the absolute value).
pub struct AbsoluteAndRelativeReserve<AbsoluteMultiLocation>(PhantomData<AbsoluteMultiLocation>);
impl<AbsoluteMultiLocation> Reserve for AbsoluteAndRelativeReserve<AbsoluteMultiLocation>
where
	AbsoluteMultiLocation: Get<MultiLocation>,
{
	fn reserve(asset: &MultiAsset) -> Option<MultiLocation> {
		RelativeReserveProvider::reserve(asset).map(|relative_reserve| {
			if relative_reserve == AbsoluteMultiLocation::get() {
				MultiLocation::here()
			} else {
				relative_reserve
			}
		})
	}
}
