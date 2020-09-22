// Copyright 2020 Parity Technologies (UK) Ltd.

//! Minimal Pallet that injects a ParachainId into Runtime storage from

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, traits::Get};

use cumulus_primitives::ParaId;

/// Configuration trait of this pallet.
pub trait Trait: frame_system::Trait {}

impl <T: Trait> Get<ParaId> for Module<T> {
	fn get() -> ParaId {
		Self::parachain_id()
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as ParachainUpgrade {
		ParachainId get(fn parachain_id) config(): ParaId = 200.into();
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {}
}
