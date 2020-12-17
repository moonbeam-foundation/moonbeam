#![cfg(test)]

use crate::*;
use std::cell::RefCell;
use frame_support::{
	assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight,
	impl_outer_event, traits::{Contains}
};
use sp_core::H256;
use sp_runtime::{
	impl_opaque_keys,
	traits::{Extrinsic as ExtrinsicT, IdentityLookup, OpaqueKeys, Verify},
	testing::{Header,UintAuthorityId,TestXt,TestSignature},
	Perbill,
	RuntimeAppPublic,
};

use sp_core::crypto::{KeyTypeId,key_types::DUMMY};
pub struct TestAuthId;
impl frame_system::offchain::AppCrypto<UintAuthorityId, TestSignature> for TestAuthId {
	type RuntimeAppPublic = UintAuthorityId;
	type GenericPublic = UintAuthorityId;
	type GenericSignature = TestSignature;
}

impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy: UintAuthorityId,
	}
}

// current error is that this doesn't implement codec::Encode but it does???
type Extrinsic = TestXt<Call<Test>, ()>;
pub type SessionIndex = u32;
pub type AccountId = u64;
pub type BlockNumber = u64;

impl_outer_origin! {
    pub enum Origin for Test where system = frame_system {}
}

thread_local! {
	pub static VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	pub static NEXT_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
		RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
	pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
	pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
	pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
	pub static TEST_SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
	pub static DISABLED: RefCell<bool> = RefCell::new(false);
	// Stores if `on_before_session_end` was called
	pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
}

pub struct TestShouldEndSession;
impl pallet_session::ShouldEndSession<u64> for TestShouldEndSession {
	fn should_end_session(now: u64) -> bool {
		let l = SESSION_LENGTH.with(|l| *l.borrow());
		now % l == 0 || FORCE_SESSION_END.with(|l| { let r = *l.borrow(); *l.borrow_mut() = false; r })
	}
}

pub struct TestSessionHandler;
impl pallet_session::SessionHandler<u64> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
	fn on_new_session<T: OpaqueKeys>(
		changed: bool,
		validators: &[(u64, T)],
		_queued_validators: &[(u64, T)],
	) {
		SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
		AUTHORITIES.with(|l|
			*l.borrow_mut() = validators.iter()
				.map(|(_, id)| id.get::<UintAuthorityId>(DUMMY).unwrap_or_default())
				.collect()
		);
	}
	fn on_disabled(_validator_index: usize) {
		DISABLED.with(|l| *l.borrow_mut() = true)
	}
	fn on_before_session_ending() {
		BEFORE_SESSION_END_CALLED.with(|b| *b.borrow_mut() = true);
	}
}

pub struct TestSessionManager;
impl pallet_session::SessionManager<u64> for TestSessionManager {
	fn end_session(_: SessionIndex) {}
	fn start_session(_: SessionIndex) {}
	fn new_session(_: SessionIndex) -> Option<Vec<u64>> {
		if !TEST_SESSION_CHANGED.with(|l| *l.borrow()) {
			VALIDATORS.with(|v| {
				let mut v = v.borrow_mut();
				*v = NEXT_VALIDATORS.with(|l| l.borrow().clone());
				Some(v.clone())
			})
		} else if DISABLED.with(|l| std::mem::replace(&mut *l.borrow_mut(), false)) {
			// If there was a disabled validator, underlying conditions have changed
			// so we return `Some`.
			Some(VALIDATORS.with(|v| v.borrow().clone()))
		} else {
			None
		}
	}
}

mod mb_session {
    pub use crate::*;
}

impl_outer_event! {
    pub enum Event for Test {
        frame_system<T>,
		pallet_balances<T>,
		pallet_session,
        mb_session<T>,
    }
}

// how can I change this to be compatible with account_id as u64
impl frame_system::offchain::SigningTypes for Test {
	type Public = UintAuthorityId;
	type Signature = TestSignature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test where
	Call<Test>: From<LocalCall>,
{
	type OverarchingCall = Call<Test>;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test where
	Call<Test>: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call<Test>,
		_public: UintAuthorityId,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call<Test>, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl sp_runtime::BoundToRuntimeAppPublic for Test {
	type Public = UintAuthorityId;
}

#[derive(Clone, Eq, PartialEq, codec::Encode, codec::Decode)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}
parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
}
impl pallet_session::Config for Test {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = ();
	type ShouldEndSession = TestShouldEndSession;
	type NextSessionRotation = ();
	type SessionManager = TestSessionManager;
	type SessionHandler = TestSessionHandler;
	type Keys = MockSessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}
parameter_types! {
    pub const SessionsPerEra: u8 = 10;
	pub const UnsignedPriority: u64 = 1 << 20;
	pub const ValidatorsPerSession: u8 = 10;
	pub const EpochDuration: u8 = 10;
}
impl Config for Test {
	type AuthorityId = TestAuthId;
	type Event = Event;
	type Call = Call<Test>;
	type Currency = pallet_balances::Module<Test>;
	type SessionsPerEra = SessionsPerEra;
	type UnsignedPriority = UnsignedPriority;
	type ValidatorsPerSession = ValidatorsPerSession;
	type EpochDuration = EpochDuration;
}
pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
//pub type Session = Module<Test>;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, 1000),
            (2, 100),
            (3, 100),
            (4, 100),
            (5, 100),
            (6, 100),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    let mut ext: sp_io::TestExternalities = t.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn genesis_config_works() {
    new_test_ext().execute_with(|| {
		assert!(System::events().is_empty());
		for x in 2..7 {
			assert_eq!(Balances::free_balance(&x),100);
		}
		assert_eq!(Balances::free_balance(&1),1000);
    });
}