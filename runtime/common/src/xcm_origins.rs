use frame_support::traits::Contains;
use xcm::latest::Location;
use xcm::prelude::Parachain;

pub struct AllowSiblingParachains;
impl Contains<Location> for AllowSiblingParachains {
	fn contains(location: &Location) -> bool {
		matches!(location.unpack(), (1, [Parachain(_)]))
	}
}
