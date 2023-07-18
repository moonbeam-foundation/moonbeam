// Copyright 2019-2023 PureStake Inc.
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

//! Precompile to receive GMP callbacks and forward to XCM

#![cfg_attr(not(feature = "std"), no_std)]

use enumflags2::BitFlags;
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::sp_runtime::traits::StaticLookup;
use frame_support::traits::Currency;
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{ConstU32, H160, U256};
use sp_std::marker::PhantomData;

type BalanceOf<T> = <<T as pallet_identity::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from pallet-identity
pub struct IdentityPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> IdentityPrecompile<Runtime>
where
	Runtime: pallet_identity::Config + pallet_evm::Config,
	Runtime::AccountId: Into<H160>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_identity::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
{
	#[precompile::public("addRegistrar(address)")]
	fn add_registrar(handle: &mut impl PrecompileHandle, account: Address) -> EvmResult {
		let account =
			Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(account.0));
		let call = pallet_identity::Call::<Runtime>::add_registrar { account };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setIdentity((bool, bytes)[], (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), bool, bytes, (bool, bytes), (bool, bytes))")]
	fn set_identity(
		handle: &mut impl PrecompileHandle,
		info: IdentityInfo<Runtime::MaxAdditionalFields>,
	) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::set_identity {
			info: Self::input_to_identity(info)?,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setSubs((address,(bool,bytes))[])")]
	fn set_subs(handle: &mut impl PrecompileHandle, subs: Vec<(Address, Data)>) -> EvmResult {
		let mut call_subs = vec![];
		for (i, (addr, data)) in subs.into_iter().enumerate() {
			let addr = Runtime::AddressMapping::into_account_id(addr.into());
			let data: pallet_identity::Data = data
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("index {i}")))?;
			call_subs.push((addr, data));
		}
		let call = pallet_identity::Call::<Runtime>::set_subs { subs: call_subs };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("clear_identity()")]
	fn clear_identity(handle: &mut impl PrecompileHandle) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::clear_identity {  };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("requestJudgement(uint32,uint256)")]
	fn request_judgement(handle: &mut impl PrecompileHandle, reg_index: u32, max_fee: U256) -> EvmResult {
		let max_fee = max_fee.try_into().map_err(|_| RevertReason::value_is_too_large("max_fee"))?;
		let call = pallet_identity::Call::<Runtime>::request_judgement {  
			reg_index,
			max_fee,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelRequest(uint32)")]
	fn cancel_request(handle: &mut impl PrecompileHandle, reg_index: u32) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::cancel_request {  
			reg_index,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setFee(uint32,uint256)")]
	fn set_fee(handle: &mut impl PrecompileHandle, index: u32, fee: U256) -> EvmResult {
		let fee = fee.try_into().map_err(|_| RevertReason::value_is_too_large("fee"))?;
		let call = pallet_identity::Call::<Runtime>::set_fee {  
			index,
			fee,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setAccountId(uint32,address)")]
	fn set_account_id(handle: &mut impl PrecompileHandle, index: u32, new: Address) -> EvmResult {
		let new = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(new.0));
		let call = pallet_identity::Call::<Runtime>::set_account_id {  
			index,
			new,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setFields(uint32,uint64)")]
	fn set_fields(handle: &mut impl PrecompileHandle, index: u32, fields: u64) -> EvmResult {
		let bit_flags = BitFlags::<pallet_identity::IdentityField>::from_bits(fields).map_err(|_| RevertReason::custom("invalid flag").in_field("fields"))?;
		let fields = pallet_identity::IdentityFields(bit_flags);
		let call = pallet_identity::Call::<Runtime>::set_fields {  
			index,
			fields,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	fn input_to_identity(
		info: IdentityInfo<Runtime::MaxAdditionalFields>,
	) -> MayRevert<Box<pallet_identity::IdentityInfo<Runtime::MaxAdditionalFields>>> {
		// let additional: Vec<(pallet_identity::Data, pallet_identity::Data)> = info.additional.into();
		let mut additional: sp_runtime::BoundedVec<
			(pallet_identity::Data, pallet_identity::Data),
			Runtime::MaxAdditionalFields,
		> = Default::default();
		let iter: Vec<_> = info.additional.into();
		for (i, (k, v)) in iter.into_iter().enumerate() {
			let k: pallet_identity::Data = k
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("additional.{i}.key")))?;
			let v: pallet_identity::Data = v
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("additional.{i}.value")))?;
			additional.try_push((k, v)).map_err(|_| {
				RevertReason::custom("out of bounds").in_field(format!("additional"))
			})?;
		}

		let pgp_fingerprint: Option<[u8; 20]> = if info.has_pgp_fingerprint {
			let fingerprint: Vec<_> = info.pgp_fingerprint.into();
			let fingerprint = fingerprint
				.try_into()
				.map_err(|_| RevertReason::custom("pgp_fingerprint must be 20 bytes long"))?;
			Some(fingerprint)
		} else {
			None
		};
		let identity_info = pallet_identity::IdentityInfo::<Runtime::MaxAdditionalFields> {
			additional,
			display: info
				.display
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("display")))?,
			legal: info
				.legal
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("legal")))?,
			web: info
				.web
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("web")))?,
			riot: info
				.riot
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("riot")))?,
			email: info
				.email
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("email")))?,
			pgp_fingerprint,
			image: info
				.image
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("image")))?,
			twitter: info
				.twitter
				.try_into()
				.map_err(|e| RevertReason::custom(e).in_field(format!("twitter")))?,
		};

		Ok(Box::new(identity_info))
	}
}

#[derive(Default, Clone, solidity::Codec)]
pub struct Data {
	has_data: bool,
	value: BoundedBytes<ConstU32<32>>,
}

// ((bool, bytes)[], (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), bool, bytes, (bool, bytes), (bool, bytes))
#[derive(Default, solidity::Codec)]
pub struct IdentityInfo<FieldLimit> {
	additional: BoundedVec<(Data, Data), FieldLimit>,
	display: Data,
	legal: Data,
	web: Data,
	riot: Data,
	email: Data,
	has_pgp_fingerprint: bool,
	pgp_fingerprint: BoundedBytes<ConstU32<20>>, // validate this
	image: Data,
	twitter: Data,
}

impl TryFrom<Data> for pallet_identity::Data {
	type Error = &'static str;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if !value.has_data {
			return Ok(pallet_identity::Data::None);
		}

		let value: Vec<_> = value.value.into();
		let value: sp_runtime::BoundedVec<_, ConstU32<32>> =
			value.try_into().map_err(|_| "exceeded bounds")?;
		Ok(pallet_identity::Data::Raw(value))
	}
}
