pub mod currency {
	use node_primitives::Balance;

	pub const MILLICENTS: Balance = 1_000_000_000;
	pub const CENTS: Balance = 1_000 * MILLICENTS;
	pub const DOLLARS: Balance = 100 * CENTS;
}

pub mod time {
	use node_primitives::{Moment};
	pub const MILLISECS_PER_BLOCK: Moment = 3000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
}