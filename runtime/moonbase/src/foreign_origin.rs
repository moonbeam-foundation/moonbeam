use crate::xcm_config::LocationToAccountId;
use crate::RuntimeOrigin;
use frame_support::traits::{EnsureOrigin, EnsureOriginWithArg, Everything};
use moonbeam_core_primitives::AccountId;
use xcm::latest::Location;
use xcm_executor::traits::ConvertLocation;

// `EnsureOriginWithArg` impl for `ForeignAssetOwnerOrigin` which allows only XCM origins
// which are locations containing the class location.
pub struct ForeignAssetOwnerOrigin;
impl EnsureOriginWithArg<RuntimeOrigin, Location> for ForeignAssetOwnerOrigin {
	type Success = AccountId;

	fn try_origin(
		o: RuntimeOrigin,
		a: &Location,
	) -> core::result::Result<Self::Success, RuntimeOrigin> {
		let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())?;
		if !a.starts_with(&origin_location) {
			return Err(o);
		}
		LocationToAccountId::convert_location(&origin_location).ok_or(o)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(a: &Location) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_xcm::Origin::Xcm(a.clone()).into())
	}
}
