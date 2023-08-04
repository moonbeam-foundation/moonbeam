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
use crate::mock::*;
use crate::{
	Data, IdentityFields, IdentityInfo, Judgement, Registrar, Registration, SubsOf, SuperOf,
};
use frame_support::{assert_ok, dispatch::Dispatchable, BoundedVec};
use frame_system::RawOrigin;
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use pallet_identity::{
	Event as IdentityEvent, IdentityField, Pallet as IdentityPallet, RegistrarInfo,
};
use parity_scale_codec::Encode;
use precompile_utils::prelude::*;
use precompile_utils::testing::*;
use sp_core::{ConstU32, H160, H256, U256};
use sp_runtime::{
	traits::{Hash, PostDispatchInfoOf},
	DispatchResultWithInfo,
};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

// #[test]
// fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
// 	check_precompile_implements_solidity_interfaces(
// 		&["Identity.sol"],
// 		PCall::supports_selector,
// 	)
// }

#[test]
fn test_add_registrar_with_registrar_origin_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarAndForceOrigin,
				PCall::add_registrar {
					account: H160::from(Bob).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::RegistrarAdded { registrar_index: 0 }
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 0,
					fields: pallet_identity::IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_add_registrar_with_non_registrar_origin_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Charlie,
				PCall::add_registrar {
					account: H160::from(Bob).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				events(),
				vec![RuntimeEvent::Evm(pallet_evm::Event::ExecutedFailed {
					address: Precompile1.into()
				}),]
			);
		})
}

#[test]
fn test_set_fee_on_existing_registrar_index_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(<IdentityPallet<Runtime>>::add_registrar(
				RuntimeOrigin::root(),
				Bob.into()
			));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 100,
					fields: pallet_identity::IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_set_fee_on_non_existing_registrar_index_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
		})
}

#[test]
fn test_set_account_id_on_existing_registrar_index_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(<IdentityPallet<Runtime>>::add_registrar(
				RuntimeOrigin::root(),
				Bob.into()
			));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_account_id {
					index: 0,
					new: Address(Charlie.into()),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Charlie.into(),
					fee: 0,
					fields: pallet_identity::IdentityFields::default(),
				})]
			);
		})
}

#[test]
fn test_set_account_id_on_non_existing_registrar_index_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_account_id {
					index: 0,
					new: Address(Charlie.into()),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
		})
}

#[test]
fn test_set_fields_on_existing_registrar_index_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(<IdentityPallet<Runtime>>::add_registrar(
				RuntimeOrigin::root(),
				Bob.into()
			));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fields {
					index: 0,
					fields: IdentityField::Display as u64 | IdentityField::Web as u64,
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::registrars().to_vec(),
				vec![Some(RegistrarInfo {
					account: Bob.into(),
					fee: 0,
					fields: pallet_identity::IdentityFields(
						IdentityField::Display | IdentityField::Web
					),
				})]
			);
		})
}

#[test]
fn test_set_fields_on_non_existing_registrar_index_fails() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_fields {
					index: 0,
					fields: IdentityField::Display as u64 | IdentityField::Web as u64,
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
		})
}

#[test]
fn test_set_identity_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						legal: Data {
							has_data: true,
							value: vec![0x02].try_into().expect("succeeds"),
						},
						web: Data {
							has_data: true,
							value: vec![0x03].try_into().expect("succeeds"),
						},
						riot: Data {
							has_data: true,
							value: vec![0x04].try_into().expect("succeeds"),
						},
						email: Data {
							has_data: true,
							value: vec![0x05].try_into().expect("succeeds"),
						},
						has_pgp_fingerprint: true,
						pgp_fingerprint: [0x06; 20].try_into().expect("succeeds"),
						image: Data {
							has_data: true,
							value: vec![0x07].try_into().expect("succeeds"),
						},
						twitter: Data {
							has_data: true,
							value: vec![0x08].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				Some(
					pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
						judgements: Default::default(),
						deposit: BasicDeposit::get() as u128,
						info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
							additional: Default::default(),
							display: pallet_identity::Data::Raw(
								vec![0x01].try_into().expect("succeeds")
							),
							legal: pallet_identity::Data::Raw(
								vec![0x02].try_into().expect("succeeds")
							),
							web: pallet_identity::Data::Raw(
								vec![0x03].try_into().expect("succeeds")
							),
							riot: pallet_identity::Data::Raw(
								vec![0x04].try_into().expect("succeeds")
							),
							email: pallet_identity::Data::Raw(
								vec![0x05].try_into().expect("succeeds")
							),
							pgp_fingerprint: Some([0x06; 20].try_into().expect("succeeds")),
							image: pallet_identity::Data::Raw(
								vec![0x07].try_into().expect("succeeds")
							),
							twitter: pallet_identity::Data::Raw(
								vec![0x08].try_into().expect("succeeds")
							),
						}
					}
				),
			);
		})
}

#[test]
fn test_set_identity_works_for_already_set_identity() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						legal: Data {
							has_data: true,
							value: vec![0x02].try_into().expect("succeeds"),
						},
						web: Data {
							has_data: true,
							value: vec![0x03].try_into().expect("succeeds"),
						},
						riot: Data {
							has_data: true,
							value: vec![0x04].try_into().expect("succeeds"),
						},
						email: Data {
							has_data: true,
							value: vec![0x05].try_into().expect("succeeds"),
						},
						has_pgp_fingerprint: true,
						pgp_fingerprint: [0x06; 20].try_into().expect("succeeds"),
						image: Data {
							has_data: true,
							value: vec![0x07].try_into().expect("succeeds"),
						},
						twitter: Data {
							has_data: true,
							value: vec![0x08].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				Some(
					pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
						judgements: Default::default(),
						deposit: BasicDeposit::get() as u128,
						info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
							additional: Default::default(),
							display: pallet_identity::Data::Raw(
								vec![0x01].try_into().expect("succeeds")
							),
							legal: pallet_identity::Data::Raw(
								vec![0x02].try_into().expect("succeeds")
							),
							web: pallet_identity::Data::Raw(
								vec![0x03].try_into().expect("succeeds")
							),
							riot: pallet_identity::Data::Raw(
								vec![0x04].try_into().expect("succeeds")
							),
							email: pallet_identity::Data::Raw(
								vec![0x05].try_into().expect("succeeds")
							),
							pgp_fingerprint: Some([0x06; 20].try_into().expect("succeeds")),
							image: pallet_identity::Data::Raw(
								vec![0x07].try_into().expect("succeeds")
							),
							twitter: pallet_identity::Data::Raw(
								vec![0x08].try_into().expect("succeeds")
							),
						}
					}
				),
			);

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0xff].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				Some(
					pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
						judgements: Default::default(),
						deposit: BasicDeposit::get() as u128,
						info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
							additional: Default::default(),
							display: pallet_identity::Data::Raw(
								vec![0xff].try_into().expect("succeeds")
							),
							legal: pallet_identity::Data::None,
							web: pallet_identity::Data::None,
							riot: pallet_identity::Data::None,
							email: pallet_identity::Data::None,
							pgp_fingerprint: None,
							image: pallet_identity::Data::None,
							twitter: pallet_identity::Data::None,
						}
					}
				),
			);
		})
}

#[test]
fn test_set_subs_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				Some(
					pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
						judgements: Default::default(),
						deposit: BasicDeposit::get() as u128,
						info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
							additional: Default::default(),
							display: pallet_identity::Data::Raw(
								vec![0x01].try_into().expect("succeeds")
							),
							legal: pallet_identity::Data::None,
							web: pallet_identity::Data::None,
							riot: pallet_identity::Data::None,
							email: pallet_identity::Data::None,
							pgp_fingerprint: None,
							image: pallet_identity::Data::None,
							twitter: pallet_identity::Data::None,
						}
					}
				),
			);

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_subs {
					subs: vec![
						(
							Address(Charlie.into()),
							Data {
								has_data: true,
								value: vec![0x01].try_into().expect("succeeds"),
							}
						),
						(
							Address(David.into()),
							Data {
								has_data: true,
								value: vec![0x02].try_into().expect("succeeds"),
							}
						)
					]
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::subs_of(AccountId::from(Bob)),
				(
					SubAccountDeposit::get() as u128 * 2,
					vec![Charlie.into(), David.into(),]
						.try_into()
						.expect("succeeds")
				),
			);
		})
}

#[test]
fn test_set_subs_fails_if_identity_not_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_subs {
					subs: vec![
						(
							Address(Charlie.into()),
							Data {
								has_data: true,
								value: vec![0x01].try_into().expect("succeeds"),
							}
						),
						(
							Address(David.into()),
							Data {
								has_data: true,
								value: vec![0x02].try_into().expect("succeeds"),
							}
						)
					]
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				events(),
				vec![RuntimeEvent::Evm(pallet_evm::Event::ExecutedFailed {
					address: Precompile1.into()
				}),]
			);
		})
}

#[test]
fn test_clear_identity_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				Some(
					pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
						judgements: Default::default(),
						deposit: BasicDeposit::get() as u128,
						info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
							additional: Default::default(),
							display: pallet_identity::Data::Raw(
								vec![0x01].try_into().expect("succeeds")
							),
							legal: pallet_identity::Data::None,
							web: pallet_identity::Data::None,
							riot: pallet_identity::Data::None,
							email: pallet_identity::Data::None,
							pgp_fingerprint: None,
							image: pallet_identity::Data::None,
							twitter: pallet_identity::Data::None,
						}
					}
				),
			);

			assert_ok!(
				RuntimeCall::Evm(evm_call(Bob, PCall::clear_identity {}.into()))
					.dispatch(RuntimeOrigin::root())
			);

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::IdentityCleared {
					who: Bob.into(),
					deposit: BasicDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				None,
			);
		})
}

#[test]
fn test_clear_identity_fails_if_no_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			assert_ok!(
				RuntimeCall::Evm(evm_call(Bob, PCall::clear_identity {}.into()))
					.dispatch(RuntimeOrigin::root())
			);

			assert_eq!(
				events(),
				vec![RuntimeEvent::Evm(pallet_evm::Event::ExecutedFailed {
					address: Precompile1.into()
				}),]
			);
		})
}

#[test]
fn test_request_judgement_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// add Alice as registrar
			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarAndForceOrigin,
				PCall::add_registrar {
					account: H160::from(Alice).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
			assert_ok!(RuntimeCall::Evm(evm_call(
				Alice,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::request_judgement {
					reg_index: 0,
					max_fee: 1000u64.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::JudgementRequested {
					who: Bob.into(),
					registrar_index: 0,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob))
					.expect("exists")
					.judgements
					.to_vec(),
				vec![(0, pallet_identity::Judgement::FeePaid(100))],
			);
		})
}

#[test]
fn test_cancel_request_works_if_identity_judgement_requested() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// add Alice as registrar
			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarAndForceOrigin,
				PCall::add_registrar {
					account: H160::from(Alice).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
			assert_ok!(RuntimeCall::Evm(evm_call(
				Alice,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			// Request judgement
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::request_judgement {
					reg_index: 0,
					max_fee: 1000u64.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::cancel_request { reg_index: 0 }.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::JudgementUnrequested {
					who: Bob.into(),
					registrar_index: 0,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob))
					.expect("exists")
					.judgements
					.to_vec(),
				vec![],
			);
		})
}

#[test]
fn test_provide_judgement_works_if_identity_judgement_requested() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// add Alice as registrar
			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarAndForceOrigin,
				PCall::add_registrar {
					account: H160::from(Alice).into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));
			assert_ok!(RuntimeCall::Evm(evm_call(
				Alice,
				PCall::set_fee {
					index: 0,
					fee: 100.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			// Request judgement
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::request_judgement {
					reg_index: 0,
					max_fee: 1000u64.into(),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			let identity =
				pallet_identity::Registration::<Balance, MaxRegistrars, MaxAdditionalFields> {
					judgements: Default::default(),
					deposit: BasicDeposit::get() as u128,
					info: pallet_identity::IdentityInfo::<MaxAdditionalFields> {
						additional: Default::default(),
						display: pallet_identity::Data::Raw(
							vec![0x01].try_into().expect("succeeds"),
						),
						legal: pallet_identity::Data::None,
						web: pallet_identity::Data::None,
						riot: pallet_identity::Data::None,
						email: pallet_identity::Data::None,
						pgp_fingerprint: None,
						image: pallet_identity::Data::None,
						twitter: pallet_identity::Data::None,
					},
				};

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob))
					.expect("")
					.info,
				identity.info
			);

			assert_ok!(RuntimeCall::Evm(evm_call(
				Alice,
				PCall::provide_judgement {
					reg_index: 0,
					target: Address(Bob.into()),
					judgement: Judgement {
						is_reasonable: true,
						..Default::default()
					},
					identity: <Runtime as frame_system::Config>::Hashing::hash_of(&identity.info),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::JudgementGiven {
					target: Bob.into(),
					registrar_index: 0,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob))
					.expect("exists")
					.judgements
					.to_vec(),
				vec![(0, pallet_identity::Judgement::Reasonable)],
			);
		})
}

#[test]
fn test_kill_identity_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				RegistrarAndForceOrigin,
				PCall::kill_identity {
					target: Address(Bob.into()),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::IdentityKilled {
					who: Bob.into(),
					deposit: BasicDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::identity(AccountId::from(Bob)),
				None,
			);
		})
}

#[test]
fn test_add_sub_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::add_sub {
					sub: Address(Charlie.into()),
					data: Data {
						has_data: true,
						value: vec![0x01].try_into().expect("succeeds"),
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::SubIdentityAdded {
					sub: Charlie.into(),
					main: Bob.into(),
					deposit: SubAccountDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::subs_of(AccountId::from(Bob)),
				(
					SubAccountDeposit::get() as u128,
					vec![Charlie.into()].try_into().expect("succeeds")
				),
			);
		})
}

#[test]
fn test_rename_sub_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::add_sub {
					sub: Address(Charlie.into()),
					data: Data {
						has_data: true,
						value: vec![0xff].try_into().expect("succeeds"),
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::rename_sub {
					sub: Address(Charlie.into()),
					data: Data {
						has_data: true,
						value: vec![0xaa].try_into().expect("succeeds"),
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::SubIdentityAdded {
					sub: Charlie.into(),
					main: Bob.into(),
					deposit: SubAccountDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::super_of(AccountId::from(Charlie)),
				Some((
					AccountId::from(Bob),
					pallet_identity::Data::Raw(vec![0xaa].try_into().expect("succeeds"))
				)),
			);
		})
}

#[test]
fn test_remove_sub_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::add_sub {
					sub: Address(Charlie.into()),
					data: Data {
						has_data: true,
						value: vec![0xff].try_into().expect("succeeds"),
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::remove_sub {
					sub: Address(Charlie.into()),
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::SubIdentityRemoved {
					sub: Charlie.into(),
					main: Bob.into(),
					deposit: SubAccountDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::super_of(AccountId::from(Charlie)),
				None,
			);
		})
}

#[test]
fn test_quit_sub_works_if_identity_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Set Bob's identity
			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::set_identity {
					info: IdentityInfo {
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						..Default::default()
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(RuntimeCall::Evm(evm_call(
				Bob,
				PCall::add_sub {
					sub: Address(Charlie.into()),
					data: Data {
						has_data: true,
						value: vec![0xff].try_into().expect("succeeds"),
					},
				}
				.into()
			))
			.dispatch(RuntimeOrigin::root()));

			assert_ok!(
				RuntimeCall::Evm(evm_call(Charlie, PCall::quit_sub {}.into()))
					.dispatch(RuntimeOrigin::root())
			);

			assert!(events().contains(&Into::<crate::mock::RuntimeEvent>::into(
				IdentityEvent::SubIdentityRevoked {
					sub: Charlie.into(),
					main: Bob.into(),
					deposit: SubAccountDeposit::get() as u128,
				}
			)));

			assert_eq!(
				<IdentityPallet<Runtime>>::super_of(AccountId::from(Charlie)),
				None,
			);
		})
}

#[test]
fn test_identity_returns_valid_data() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			// assert_ok!(RuntimeCall::Evm(evm_call(
			// 	Bob,
			// 	PCall::set_identity {
			// 		info: IdentityInfo {
			// 			display: Data {
			// 				has_data: true,
			// 				value: vec![0x01].try_into().expect("succeeds"),
			// 			},
			// 			legal: Data {
			// 				has_data: true,
			// 				value: vec![0x02].try_into().expect("succeeds"),
			// 			},
			// 			web: Data {
			// 				has_data: true,
			// 				value: vec![0x03].try_into().expect("succeeds"),
			// 			},
			// 			riot: Data {
			// 				has_data: true,
			// 				value: vec![0x04].try_into().expect("succeeds"),
			// 			},
			// 			email: Data {
			// 				has_data: true,
			// 				value: vec![0x05].try_into().expect("succeeds"),
			// 			},
			// 			has_pgp_fingerprint: true,
			// 			pgp_fingerprint: [0x06; 20].try_into().expect("succeeds"),
			// 			image: Data {
			// 				has_data: true,
			// 				value: vec![0x07].try_into().expect("succeeds"),
			// 			},
			// 			twitter: Data {
			// 				has_data: true,
			// 				value: vec![0x08].try_into().expect("succeeds"),
			// 			},
			// 		},
			// 	}
			// 	.into()
			// ))
			// .dispatch(RuntimeOrigin::root()));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::identity {
						who: H160::from(Alice).into(),
					},
				)
				.expect_no_logs()
				.execute_returns(Registration {
					is_valid: true,
					judgements: vec![],
					deposit: BasicDeposit::get().into(),
					info: IdentityInfo::<MaxAdditionalFields> {
						additional: vec![].try_into().expect("succeeds"),
						display: Data {
							has_data: true,
							value: vec![0x01].try_into().expect("succeeds"),
						},
						legal: Data {
							has_data: true,
							value: vec![0x02].try_into().expect("succeeds"),
						},
						web: Data {
							has_data: true,
							value: vec![0x03].try_into().expect("succeeds"),
						},
						riot: Data {
							has_data: true,
							value: vec![0x04].try_into().expect("succeeds"),
						},
						email: Data {
							has_data: true,
							value: vec![0x05].try_into().expect("succeeds"),
						},
						has_pgp_fingerprint: true,
						pgp_fingerprint: [0x06; 20].try_into().expect("succeeds"),
						image: Data {
							has_data: true,
							value: vec![0x07].try_into().expect("succeeds"),
						},
						twitter: Data {
							has_data: true,
							value: vec![0x08].try_into().expect("succeeds"),
						},
					},
				});
		})
}

#[test]
fn test_identity_returns_none_if_not_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000), (Bob.into(), 100_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::identity {
						who: H160::from(Alice).into(),
					},
				)
				.expect_no_logs()
				.execute_returns(Registration::<MaxAdditionalFields>::default());
		})
}

fn p() {
	println!("{:?}", events());
}
