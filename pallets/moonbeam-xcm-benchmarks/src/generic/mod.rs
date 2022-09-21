pub use pallet::*;

pub mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + crate::Config + pallet_xcm_benchmarks::generic::Config
	{
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);
}
