//TODO license

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use pallet_author_mapping::{migrations::TwoXToBlake, Config as AuthorMappingConfig};
use pallet_migrations::{Config as MigrationsConfig, Migration};
use sp_std::marker::PhantomData;

/// A moonbeam migration wrapping the similarly named migration in pallet-author-mapping
pub struct AuthorMappingTwoXToBlake<T>(PhantomData<T>);

impl<T: AuthorMappingConfig + MigrationsConfig> Migration for AuthorMappingTwoXToBlake<T> {
	fn friendly_name(&self) -> &str {
		"MM_Author_Mapping_TwoXToBlake"
	}

	fn migrate(&self, available_weight: Weight) -> Weight {
		//TODO @notlesh, what should I be doing with the available_weight here
		TwoXToBlake::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::post_upgrade()
	}
}
