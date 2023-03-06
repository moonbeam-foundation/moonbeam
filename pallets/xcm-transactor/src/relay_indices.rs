use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

// TODO: function that takes input call and

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct RelayChainIndices {
	pub pallets: PalletIndices,
	pub calls: CallIndices,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct PalletIndices {
	pub staking: u8,
	pub utility: u8,
	pub hrmp: u8,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct CallIndices {
	pub staking: StakingIndices,
	pub utility: UtilityIndices,
	pub hrmp: HrmpIndices,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct UtilityIndices {
	pub as_derivative: u8,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct StakingIndices {
	pub bond: u8,
	pub bond_extra: u8,
	pub unbond: u8,
	pub withdraw_unbonded: u8,
	pub validate: u8,
	pub nominate: u8,
	pub chill: u8,
	pub set_payee: u8,
	pub set_controller: u8,
	pub rebond: u8,
}

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct HrmpIndices {
	pub init_open_channel: u8,
	pub accept_open_channel: u8,
	pub close_channel: u8,
}
