#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::offchain::{http, Duration},
	};
	use frame_system::pallet_prelude::BlockNumberFor;
	use lite_json::JsonValue;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_ethereum::Config {
		/// Overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		SubmittedToCelestia {
			height: u64,
			namespace: Vec<u8>,
			commitment: Vec<u8>,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// TODO: Use `Local Storage` API to coordinate runs of the worker.
		/// There is no guarantee for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			log::info!("==== offchain worker ====");
			if let Some(eth_block) = pallet_ethereum::CurrentBlock::<T>::get() {
				let _encoded = eth_block.encode();
				// TODO: Post to celestia
			}
			Self::deposit_event(Event::SubmittedToCelestia {
				height: 190,
				namespace: Vec::default(),
				commitment: Vec::default(),
			});
			// match Self::fetch_price() {
			// 	Err(_) => log::error!("Error posting to celestia"),
			// 	_ => Self::deposit_event(Event::SubmittedToCelestia {
			// 		height: 190,
			// 		namespace: Vec::default(),
			// 		commitment: Vec::default(),
			// 	}),
			// }
		}
	}

	impl<T: Config> Pallet<T> {
		/// Fetch current price and return the result in cents.
		fn fetch_price() -> Result<u32, http::Error> {
			// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
			// deadline to 2s to complete the external call.
			// You can also wait indefinitely for the response, however you may still get a timeout
			// coming from the host machine.
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
			// Initiate an external HTTP GET request.
			// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
			// you can find in `sp_io`. The API is trying to be similar to `request`, but
			// since we are running in a custom WASM execution environment we can't simply
			// import the library here.
			let request = http::Request::get(
				"https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD",
			);
			// We set the deadline for sending of the request, note that awaiting response can
			// have a separate deadline. Next we send the request, before that it's also possible
			// to alter request headers or stream body content in case of non-GET requests.
			let pending = request
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;

			// The request is already being processed by the host, we are free to do anything
			// else in the worker (we can send multiple concurrent requests too).
			// At some point however we probably want to check the response though,
			// so we can block current thread and wait for it to finish.
			// Note that since the request is being driven by the host, we don't have to wait
			// for the request to have it complete, we will just not read the response.
			let response = pending
				.try_wait(deadline)
				.map_err(|_| http::Error::DeadlineReached)??;
			// Let's check the status code before we proceed to reading the response.
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown);
			}

			// Next we want to fully read the response body and collect it to a vector of bytes.
			// Note that the return object allows you to read the body in chunks as well
			// with a way to control the deadline.
			let body: Vec<u8> = response.body().collect::<Vec<u8>>();

			// Create a str slice from the body.
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			let price = match Self::parse_price(body_str) {
				Some(price) => Ok(price),
				None => {
					log::warn!("Unable to extract price from the response: {:?}", body_str);
					Err(http::Error::Unknown)
				}
			}?;

			log::warn!("Got price: {} cents", price);

			Ok(price)
		}

		/// Parse the price from the given JSON string using `lite-json`.
		///
		/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
		fn parse_price(price_str: &str) -> Option<u32> {
			// {"USD":41947.49}
			let val = lite_json::parse_json(price_str);
			let price = match val.ok()? {
				JsonValue::Object(obj) => {
					let (_, v) = obj
						.into_iter()
						.find(|(k, _)| k.iter().copied().eq("USD".chars()))?;
					match v {
						JsonValue::Number(number) => number,
						_ => return None,
					}
				}
				_ => return None,
			};

			let exp = price.fraction_length.saturating_sub(2);
			Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
		}
	}
}
