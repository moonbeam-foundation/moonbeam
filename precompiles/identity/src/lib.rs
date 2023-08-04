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
use sp_core::{ConstU32, H160, H256, U256};
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

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
	Runtime::Hash: From<H256>,
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

	#[precompile::public("setIdentity((((bool,bytes),(bool,bytes))[],(bool,bytes),(bool,bytes),(bool,bytes),(bool,bytes),(bool,bytes),bool,bytes,(bool,bytes),(bool,bytes)))")]
	fn set_identity(
		handle: &mut impl PrecompileHandle,
		info: IdentityInfo<Runtime::MaxAdditionalFields>,
	) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::set_identity {
			info: Self::identity_to_input(info)?,
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

	#[precompile::public("clearIdentity()")]
	fn clear_identity(handle: &mut impl PrecompileHandle) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::clear_identity {};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("requestJudgement(uint32,uint256)")]
	fn request_judgement(
		handle: &mut impl PrecompileHandle,
		reg_index: u32,
		max_fee: U256,
	) -> EvmResult {
		let max_fee = max_fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("max_fee"))?;
		let call = pallet_identity::Call::<Runtime>::request_judgement { reg_index, max_fee };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("cancelRequest(uint32)")]
	fn cancel_request(handle: &mut impl PrecompileHandle, reg_index: u32) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::cancel_request { reg_index };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setFee(uint32,uint256)")]
	fn set_fee(handle: &mut impl PrecompileHandle, index: u32, fee: U256) -> EvmResult {
		let fee = fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("fee"))?;
		let call = pallet_identity::Call::<Runtime>::set_fee { index, fee };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setAccountId(uint32,address)")]
	fn set_account_id(handle: &mut impl PrecompileHandle, index: u32, new: Address) -> EvmResult {
		let new = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(new.0));
		let call = pallet_identity::Call::<Runtime>::set_account_id { index, new };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("setFields(uint32,uint64)")]
	fn set_fields(handle: &mut impl PrecompileHandle, index: u32, fields: u64) -> EvmResult {
		let bit_flags = BitFlags::<pallet_identity::IdentityField>::from_bits(fields)
			.map_err(|_| RevertReason::custom("invalid flag").in_field("fields"))?;
		let fields = pallet_identity::IdentityFields(bit_flags);
		let call = pallet_identity::Call::<Runtime>::set_fields { index, fields };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public(
		"provideJudgement(uint32,address,(bool,bool,uint256,bool,bool,bool,bool,bool),bytes32)"
	)]
	fn provide_judgement(
		handle: &mut impl PrecompileHandle,
		reg_index: u32,
		target: Address,
		judgement: Judgement,
		identity: H256,
	) -> EvmResult {
		let target = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(target.0));
		let judgement = Self::judgment_to_input(judgement)?;
		let identity: Runtime::Hash = identity.into();
		let call = pallet_identity::Call::<Runtime>::provide_judgement {
			reg_index,
			target,
			judgement,
			identity,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("killIdentity(address)")]
	fn kill_identity(handle: &mut impl PrecompileHandle, target: Address) -> EvmResult {
		let target = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(target.0));
		let call = pallet_identity::Call::<Runtime>::kill_identity { target };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("addSub(address,(bool,bytes))")]
	fn add_sub(handle: &mut impl PrecompileHandle, sub: Address, data: Data) -> EvmResult {
		let sub = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(sub.0));
		let data: pallet_identity::Data = data
			.try_into()
			.map_err(|e| RevertReason::custom(e).in_field("data"))?;
		let call = pallet_identity::Call::<Runtime>::add_sub { sub, data };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("renameSub(address,(bool,bytes))")]
	fn rename_sub(handle: &mut impl PrecompileHandle, sub: Address, data: Data) -> EvmResult {
		let sub = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(sub.0));
		let data: pallet_identity::Data = data
			.try_into()
			.map_err(|e| RevertReason::custom(e).in_field("data"))?;
		let call = pallet_identity::Call::<Runtime>::rename_sub { sub, data };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("removeSub(address)")]
	fn remove_sub(handle: &mut impl PrecompileHandle, sub: Address) -> EvmResult {
		let sub = Runtime::Lookup::unlookup(Runtime::AddressMapping::into_account_id(sub.0));
		let call = pallet_identity::Call::<Runtime>::remove_sub { sub };

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("quitSub()")]
	fn quit_sub(handle: &mut impl PrecompileHandle) -> EvmResult {
		let call = pallet_identity::Call::<Runtime>::quit_sub {};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("identity(address)")]
	#[precompile::view]
	fn identity(
		handle: &mut impl PrecompileHandle,
		who: Address,
	) -> EvmResult<Registration<Runtime::MaxAdditionalFields>> {
		// Storage item: IdentityOf:
		// BoundedVec((RegistrarIndex(4) + Judgement(17)) * MaxRegistrars(20)) + Balance(16) + IdentityInfo(BoundedVec(Data(33) * MaxAdditionalFields(100)) + Data(33)*7 + 20)
		handle.record_db_read::<Runtime>(3987)?;

		let who: H160 = who.into();
		let who = Runtime::AddressMapping::into_account_id(who);
		let identity = pallet_identity::Pallet::<Runtime>::identity(who);

		Ok(Self::identity_to_output(identity)?)
	}

	#[precompile::public("superOf(address)")]
	#[precompile::view]
	fn super_of(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<SuperOf> {
		// Storage item: SuperOf:
		// AccountId(20) + Data(33)
		handle.record_db_read::<Runtime>(53)?;

		let who: H160 = who.into();
		let who = Runtime::AddressMapping::into_account_id(who);
		if let Some((account, data)) = pallet_identity::Pallet::<Runtime>::super_of(who) {
			Ok(SuperOf {
				is_valid: true,
				account: Address(account.into()),
				data: Self::data_to_output(data),
			})
		} else {
			Ok(SuperOf::default())
		}
	}

	#[precompile::public("subsOf(address)")]
	#[precompile::view]
	fn subs_of(handle: &mut impl PrecompileHandle, who: Address) -> EvmResult<SubsOf> {
		// Storage item: SubsOf:
		// BoundedVec(AccountId(20) * MaxSubAccounts(100))
		handle.record_db_read::<Runtime>(2000)?;

		let who: H160 = who.into();
		let who = Runtime::AddressMapping::into_account_id(who);
		let (balance, accounts) = pallet_identity::Pallet::<Runtime>::subs_of(who);

		let accounts = accounts
			.into_iter()
			.map(|account| Address(account.into()))
			.collect();

		Ok(SubsOf {
			balance: balance.into(),
			accounts,
		})
	}

	#[precompile::public("registrars()")]
	#[precompile::view]
	fn registrars(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Registrar>> {
		// Storage item: Registrars:
		// BoundedVec((AccountId(20) + Balance(16) + IdentityFields(8)) * MaxSubAccounts(100))
		handle.record_db_read::<Runtime>(4400)?;

		let registrars = pallet_identity::Pallet::<Runtime>::registrars()
			.into_iter()
			.enumerate()
			.map(|(index, maybe_reg)| {
				if let Some(reg) = maybe_reg {
					Registrar {
						index: index as u32,
						is_valid: true,
						account: Address(reg.account.into()),
						fee: reg.fee.into(),
						fields: IdentityFields {
							display: reg
								.fields
								.0
								.contains(pallet_identity::IdentityField::Display),
							legal: reg.fields.0.contains(pallet_identity::IdentityField::Legal),
							web: reg.fields.0.contains(pallet_identity::IdentityField::Web),
							riot: reg.fields.0.contains(pallet_identity::IdentityField::Riot),
							email: reg.fields.0.contains(pallet_identity::IdentityField::Email),
							pgp_fingerprint: reg
								.fields
								.0
								.contains(pallet_identity::IdentityField::PgpFingerprint),
							image: reg.fields.0.contains(pallet_identity::IdentityField::Image),
							twitter: reg
								.fields
								.0
								.contains(pallet_identity::IdentityField::Twitter),
						},
					}
				} else {
					Registrar {
						index: index as u32,
						is_valid: false,
						..Default::default()
					}
				}
			})
			.collect();

		Ok(registrars)
	}

	fn identity_to_input(
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

	fn identity_to_output(
		registration: Option<
			pallet_identity::Registration<
				BalanceOf<Runtime>,
				Runtime::MaxRegistrars,
				Runtime::MaxAdditionalFields,
			>,
		>,
	) -> MayRevert<Registration<Runtime::MaxAdditionalFields>> {
		if registration.is_none() {
			return Ok(Registration::<Runtime::MaxAdditionalFields>::default());
		}

		let registration = registration.expect("none case checked above; qed");
		let mut identity_info = IdentityInfo::<Runtime::MaxAdditionalFields> {
			additional: Default::default(),
			display: Self::data_to_output(registration.info.display),
			legal: Self::data_to_output(registration.info.legal),
			web: Self::data_to_output(registration.info.web),
			riot: Self::data_to_output(registration.info.riot),
			email: Self::data_to_output(registration.info.email),
			has_pgp_fingerprint: false,
			pgp_fingerprint: Default::default(),
			image: Self::data_to_output(registration.info.image),
			twitter: Self::data_to_output(registration.info.twitter),
		};

		let mut additional = vec![];
		for (k, v) in registration.info.additional.into_iter() {
			let k: Data = Self::data_to_output(k);
			let v: Data = Self::data_to_output(v);
			additional.push((k, v));
		}

		if let Some(pgp_fingerprint) = registration.info.pgp_fingerprint {
			identity_info.has_pgp_fingerprint = true;
			identity_info.pgp_fingerprint = pgp_fingerprint.into();
		}

		identity_info.additional = additional.into();

		let mut judgements = vec![];
		for (index, judgement) in registration.judgements.into_iter() {
			judgements.push((index, Self::judgement_to_output(judgement)));
		}

		let reg = Registration::<Runtime::MaxAdditionalFields> {
			is_valid: true,
			judgements: judgements.into(),
			deposit: registration.deposit.into(),
			info: identity_info,
		};

		Ok(reg)
	}

	fn judgement_to_output(value: pallet_identity::Judgement<BalanceOf<Runtime>>) -> Judgement {
		let mut judgement = Judgement::default();

		match value {
			pallet_identity::Judgement::Unknown => {
				judgement.is_unknown = true;
			}
			pallet_identity::Judgement::FeePaid(balance) => {
				judgement.is_fee_paid = true;
				judgement.fee_paid_amount = balance.into();
			}
			pallet_identity::Judgement::Reasonable => {
				judgement.is_reasonable = true;
			}
			pallet_identity::Judgement::KnownGood => {
				judgement.is_known_good = true;
			}
			pallet_identity::Judgement::OutOfDate => {
				judgement.is_out_of_date = true;
			}
			pallet_identity::Judgement::LowQuality => {
				judgement.is_low_quality = true;
			}
			pallet_identity::Judgement::Erroneous => {
				judgement.is_erroneous = true;
			}
		};

		judgement
	}

	fn judgment_to_input(
		value: Judgement,
	) -> Result<pallet_identity::Judgement<BalanceOf<Runtime>>, RevertReason> {
		if value.is_unknown {
			return Ok(pallet_identity::Judgement::Unknown);
		}

		if value.is_fee_paid {
			let amount: BalanceOf<Runtime> = value
				.fee_paid_amount
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("fee_paid_amount").into())?;

			return Ok(pallet_identity::Judgement::FeePaid(amount));
		}

		if value.is_reasonable {
			return Ok(pallet_identity::Judgement::Reasonable);
		}

		if value.is_known_good {
			return Ok(pallet_identity::Judgement::KnownGood);
		}

		if value.is_out_of_date {
			return Ok(pallet_identity::Judgement::OutOfDate);
		}

		if value.is_low_quality {
			return Ok(pallet_identity::Judgement::LowQuality);
		}

		if value.is_erroneous {
			return Ok(pallet_identity::Judgement::Erroneous);
		}

		return Err(RevertReason::custom("invalid"));
	}

	fn data_to_output(data: pallet_identity::Data) -> Data {
		let mut output = Data::default();
		match data {
			pallet_identity::Data::None => (),
			pallet_identity::Data::Raw(bytes) => {
				let bytes: Vec<_> = bytes.into();
				output.has_data = true;
				output.value = bytes.into();
			}
			pallet_identity::Data::BlakeTwo256(bytes) => {
				output.has_data = true;
				output.value = bytes.into();
			}
			pallet_identity::Data::Sha256(bytes) => {
				output.has_data = true;
				output.value = bytes.into();
			}
			pallet_identity::Data::Keccak256(bytes) => {
				output.has_data = true;
				output.value = bytes.into();
			}
			pallet_identity::Data::ShaThree256(bytes) => {
				output.has_data = true;
				output.value = bytes.into();
			}
		}

		output
	}
}

#[derive(Default, Debug, Eq, PartialEq, solidity::Codec)]
pub struct Data {
	has_data: bool,
	value: BoundedBytes<ConstU32<32>>,
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

// ((bool, bytes)[], (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), (bool, bytes), bool, bytes, (bool, bytes), (bool, bytes))
#[derive(Eq, PartialEq, Debug, solidity::Codec)]
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

impl<T> Default for IdentityInfo<T> {
	fn default() -> Self {
		Self {
			additional: Default::default(),
			display: Default::default(),
			legal: Default::default(),
			web: Default::default(),
			riot: Default::default(),
			email: Default::default(),
			has_pgp_fingerprint: Default::default(),
			pgp_fingerprint: Default::default(),
			image: Default::default(),
			twitter: Default::default(),
		}
	}
}

// impl<T> Clone for IdentityInfo<T> {
// 	fn clone(&self) -> Self {
// 		let additional: BoundedVec::<(Data, Data), T> = self.additional.as_byte_slice().into();

// 		Self {
// 			additional,
// 			display: self.display.clone(),
// 			legal: self.legal.clone(),
// 			web: self.web.clone(),
// 			riot: self.riot.clone(),
// 			email: self.email.clone(),
// 			has_pgp_fingerprint: self.has_pgp_fingerprint.clone(),
// 			pgp_fingerprint: self.pgp_fingerprint.clone(),
// 			image: self.image.clone(),
// 			twitter: self.twitter.clone(),
// 		}
// 	}
// }

// (bool, bool, uint256, bool, bool, bool, bool, bool)
#[derive(Eq, PartialEq, Default, Debug, solidity::Codec)]
pub struct Judgement {
	is_unknown: bool,
	is_fee_paid: bool,
	fee_paid_amount: U256,
	is_reasonable: bool,
	is_known_good: bool,
	is_out_of_date: bool,
	is_low_quality: bool,
	is_erroneous: bool,
}

#[derive(Eq, PartialEq, Debug, solidity::Codec)]
pub struct Registration<FieldLimit> {
	is_valid: bool,
	judgements: Vec<(u32, Judgement)>,
	deposit: U256,
	info: IdentityInfo<FieldLimit>,
}

impl<T> Default for Registration<T> {
	fn default() -> Self {
		Self {
			is_valid: false,
			judgements: vec![],
			deposit: Default::default(),
			info: Default::default(),
		}
	}
}

#[derive(Default, Debug, solidity::Codec)]
pub struct SuperOf {
	is_valid: bool,
	account: Address,
	data: Data,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct SubsOf {
	balance: U256,
	accounts: Vec<Address>,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct IdentityFields {
	display: bool,
	legal: bool,
	web: bool,
	riot: bool,
	email: bool,
	pgp_fingerprint: bool,
	image: bool,
	twitter: bool,
}

#[derive(Default, Debug, solidity::Codec)]
pub struct Registrar {
	index: u32,
	is_valid: bool,
	account: Address,
	fee: U256,
	fields: IdentityFields,
}
