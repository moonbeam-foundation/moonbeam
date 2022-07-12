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

#[macro_export]
macro_rules! impl_moonbeam_xcm_call {
	{} => {

		pub struct MoonbeamCall;
		impl CallDispatcher<Call> for MoonbeamCall {
			fn dispatch(
				call: Call,
				origin: Origin,
			) -> Result<PostDispatchInfoOf<Call>, DispatchErrorWithPostInfo<PostDispatchInfoOf<Call>>> {
				if let Ok(raw_origin) = TryInto::<RawOrigin<AccountId>>::try_into(origin.clone().caller) {
					match (call.clone(), raw_origin) {
						(
							Call::EthereumXcm(pallet_ethereum_xcm::Call::transact { .. }),
							RawOrigin::Signed(account_id)
						) => {
							return Call::dispatch(
								call,
								pallet_ethereum_xcm::Origin::XcmEthereumTransaction(
									account_id.into()
								).into()
							);
						},
						_ => {}
					}
				}
				Call::dispatch(call, origin)
			}
		}
	}
}
