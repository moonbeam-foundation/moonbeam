// Copyright 2019-2025 PureStake Inc.
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

//! Substrate EVM tracing.
//!
//! The purpose of this crate is enable tracing the EVM opcode execution and will be used by
//! both Dapp developers - to get a granular view on their transactions - and indexers to access
//! the EVM callstack (internal transactions).
//!
//! Proxies EVM messages to the host functions.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod tracer {
	use ethereum_types::H256;
	use evm_tracing_events::{EvmEvent, GasometerEvent, RuntimeEvent, StepEventFilter};
	use parity_scale_codec::{Decode, Encode};

	use evm::tracing::{using as evm_using, EventListener as EvmListener};
	use evm_gasometer::tracing::{using as gasometer_using, EventListener as GasometerListener};
	use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener};
	use sp_runtime::DispatchError;
	use sp_std::{cell::RefCell, rc::Rc};

	/// The current EthereumXcmTransaction trace status.
	#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
	pub enum EthereumTracingStatus {
		/// A full block trace.
		Block,
		/// A single transaction.
		Transaction(H256),
		/// Exit signal.
		TransactionExited,
	}

	environmental::environmental!(ETHEREUM_TRACING_STATUS: EthereumTracingStatus);

	struct ListenerProxy<T>(pub Rc<RefCell<T>>);
	impl<T: GasometerListener> GasometerListener for ListenerProxy<T> {
		/// Forwards a gasometer tracing event to the wrapped listener.
		///
		/// This method delegates the received `evm_gasometer::tracing::Event` to the inner
		/// listener by mutably borrowing it and calling its `event` handler.
		///
		/// # Examples
		///
		/// ```
		/// use std::rc::Rc;
		/// use std::cell::RefCell;
		///
		/// struct Sink {
		///     called: bool,
		/// }
		///
		/// impl Sink {
		///     fn new() -> Self { Self { called: false } }
		/// }
		///
		/// impl evm_gasometer::tracing::GasometerListener for Sink {
		///     fn event(&mut self, _event: evm_gasometer::tracing::Event) {
		///         self.called = true;
		///     }
		/// }
		///
		/// let sink = Rc::new(RefCell::new(Sink::new()));
		/// let proxy = crate::tracer::ListenerProxy(sink.clone());
		/// // simulate an event (use a default or constructed Event appropriate for your env)
		/// let evt = evm_gasometer::tracing::Event::default();
		/// proxy.event(evt);
		/// assert!(sink.borrow().called);
		/// ```
		fn event(&mut self, event: evm_gasometer::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
		fn event(&mut self, event: evm_runtime::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	impl<T: EvmListener> EvmListener for ListenerProxy<T> {
		/// Forwards an EVM tracing event to the wrapped listener.
		///
		/// Delegates the provided `evm::tracing::Event` to the inner listener held by this proxy.
		///
		/// # Examples
		///
		/// ```no_run
		/// // Given a `ListenerProxy` wrapping a listener that implements `EvmListener`,
		/// // calling `event` forwards the event to the inner listener:
		/// // let mut proxy: ListenerProxy<YourListener> = ...;
		/// // proxy.event(evm::tracing::Event::SomeVariant);
		/// ```
		fn event(&mut self, event: evm::tracing::Event) {
			self.0.borrow_mut().event(event);
		}
	}

	pub struct EthereumTracer;

	impl EthereumTracer {
		/// Run a closure with the Ethereum tracing status set to a specific transaction hash.
		///
		/// Sets the thread-local `ETHEREUM_TRACING_STATUS` to `EthereumTracingStatus::Transaction(tx_hash)`
		/// for the duration of `func` and then restores the previous status. Returns whatever `func` returns.
		///
		/// # Examples
		///
		/// ```
		/// use ethereum_types::H256;
		/// use sp_runtime::DispatchError;
		///
		/// let tx_hash = H256::zero();
		/// let res: Result<(), DispatchError> = tracer::EthereumTracer::transaction(tx_hash, || {
		///     // code executed with tracing status set to the given transaction hash
		///     Ok(())
		/// });
		/// assert!(res.is_ok());
		/// ```
		pub fn transaction(
			tx_hash: H256,
			func: impl FnOnce() -> Result<(), DispatchError>,
		) -> Result<(), DispatchError> {
			ETHEREUM_TRACING_STATUS::using(&mut EthereumTracingStatus::Transaction(tx_hash), func)
		}

		/// Run `func` with the global Ethereum tracing status set to `Block`.
		///
		/// While `func` executes, the thread-local `ETHEREUM_TRACING_STATUS` is set to
		/// `EthereumTracingStatus::Block`. The previous status is restored after `func`
		/// returns. The function returns the `Result` produced by `func`.
		///
		/// # Examples
		///
		/// ```
		/// use crate::tracer::{EthereumTracer, EthereumTracingStatus};
		/// use sp_runtime::DispatchError;
		///
		/// let res = EthereumTracer::block(|| -> Result<(), DispatchError> {
		///     // code executed with tracing status = Block
		///     Ok(())
		/// });
		/// assert!(res.is_ok());
		/// ```
		pub fn block(
			func: impl FnOnce() -> Result<(), DispatchError>,
		) -> Result<(), DispatchError> {
			ETHEREUM_TRACING_STATUS::using(&mut EthereumTracingStatus::Block, func)
		}

		/// Mark the current tracing scope as having exited a transaction.
		///
		/// Sets the global `ETHEREUM_TRACING_STATUS` to `EthereumTracingStatus::TransactionExited`.
		///
		/// # Examples
		///
		/// ```
		/// use crate::tracer::{EthereumTracer, EthereumTracingStatus};
		///
		/// EthereumTracer::transaction_exited();
		/// assert_eq!(EthereumTracer::status(), Some(EthereumTracingStatus::TransactionExited));
		/// ```
		pub fn transaction_exited() {
			ETHEREUM_TRACING_STATUS::with(|state| {
				*state = EthereumTracingStatus::TransactionExited
			});
		}

		/// Returns the current Ethereum tracing status, if any.
		///
		/// Reads the thread-local `ETHEREUM_TRACING_STATUS` and returns a cloned copy of its
		/// current `EthereumTracingStatus` value wrapped in `Some`, or `None` if no status is set.
		///
		/// # Examples
		///
		/// ```
		/// // Query current tracing state (may be `None` when tracing is not active).
		/// if let Some(status) = tracer::status() {
		///     match status {
		///         tracer::EthereumTracingStatus::Block => { /* block tracing active */ }
		///         tracer::EthereumTracingStatus::Transaction(tx_hash) => { /* tracing tx_hash */ }
		///         tracer::EthereumTracingStatus::TransactionExited => { /* tracing ended */ }
		///     }
		/// }
		/// ```
		pub fn status() -> Option<EthereumTracingStatus> {
			ETHEREUM_TRACING_STATUS::with(|state| state.clone())
		}
	}

	pub struct EvmTracer {
		step_event_filter: StepEventFilter,
	}

	impl EvmTracer {
		pub fn new() -> Self {
			Self {
				step_event_filter: moonbeam_primitives_ext::moonbeam_ext::step_event_filter(),
			}
		}

		/// Setup event listeners and execute provided closure.
		///
		/// Consume the tracer and return it alongside the return value of
		/// the closure.
		pub fn trace<R, F: FnOnce() -> R>(self, f: F) {
			let wrapped = Rc::new(RefCell::new(self));

			let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
			let mut runtime = ListenerProxy(Rc::clone(&wrapped));
			let mut evm = ListenerProxy(Rc::clone(&wrapped));

			// Each line wraps the previous `f` into a `using` call.
			// Listening to new events results in adding one new line.
			// Order is irrelevant when registering listeners.
			let f = || runtime_using(&mut runtime, f);
			let f = || gasometer_using(&mut gasometer, f);
			let f = || evm_using(&mut evm, f);
			f();
		}

		pub fn emit_new() {
			moonbeam_primitives_ext::moonbeam_ext::call_list_new();
		}
	}

	impl EvmListener for EvmTracer {
		/// Proxies `evm::tracing::Event` to the host.
		fn event(&mut self, event: evm::tracing::Event) {
			let event: EvmEvent = event.into();
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::evm_event(message);
		}
	}

	impl GasometerListener for EvmTracer {
		/// Proxies `evm_gasometer::tracing::Event` to the host.
		fn event(&mut self, event: evm_gasometer::tracing::Event) {
			let event: GasometerEvent = event.into();
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::gasometer_event(message);
		}
	}

	impl RuntimeListener for EvmTracer {
		/// Proxies `evm_runtime::tracing::Event` to the host.
		fn event(&mut self, event: evm_runtime::tracing::Event) {
			let event = RuntimeEvent::from_evm_event(event, self.step_event_filter);
			let message = event.encode();
			moonbeam_primitives_ext::moonbeam_ext::runtime_event(message);
		}
	}
}
