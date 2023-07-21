A port crate of some of the tracing related rpc requests from the go-ethereum [debug namespace](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug). Includes `debug_traceTransaction`, `debug_traceBlockByNumber` and `debug_traceBlockByHash`.

## How tracing works in Moonbeam

Runtime wasms compiled with the `tracing` evm feature will emit events related to entering/exiting substates or opcode execution. This events are used by developers or indexer services to get a granular view on an evm transaction.

Tracing wasms for each moonbeam/river/base runtime versions live at `moonbeam-runtime-overrides` repository in github.

Tracing functionality in Moonbeam makes heavy use of [environmental](https://docs.rs/environmental/latest/environmental/):

- The rpc request must create a runtime api instance to replay the transaction. The runtime api call is made `using` `environmental`.
- Once in the wasm, the target evm transaction is replayed by calling the evm also `using` `environmental`.
- This allows:
  1. Listen to new events from the evm in the moonbeam runtime wasm.
  2. Proxy those events to the client (through a host function), which is also listening for events from the runtime.
- This way we don't make use of (limited) wasm memory, and instead store the evm emitted events content in the client.

Once the evm execution concludes, the runtime context exited and all events have been stored in the client memory, we support formatting the captured events in different ways that are convenient for the end-user, like raw format (opcode level tracing), callTracer (used as a default formatter by geth) or blockscout custom tracer.

## On Runtime Api versioning

This text aims to describe the process of adding new Runtime Api versions and supporting old ones.

### How to create a Runtime Api version

```
sp_api::decl_runtime_apis! {
	pub trait DebugRuntimeApi {
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			trace_type: single::TraceType,
		) -> Result<single::TransactionTrace, sp_runtime::DispatchError>;
	}
}
```

For the `trace_transaction` method above, we need a new header argument, and the response will no longer be a single::TransactionTrace but an empty result () because we will handle the result client side using environmental.

Becomes:

```
sp_api::decl_runtime_apis! {
	#[api_version(2)]
	pub trait DebugRuntimeApi {

		#[changed_in(2)]
		fn trace_transaction(
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			trace_type: single::TraceType,
		) -> Result<single::TransactionTrace, sp_runtime::DispatchError>;

		fn trace_transaction(
			header: &Block::Header,
			extrinsics: Vec<Block::Extrinsic>,
			transaction: &Transaction,
			trace_type: single::TraceType,
		) -> Result<(), sp_runtime::DispatchError>;
	}
}
```

Substrate provides two macro attributes to do what we want: `api_version` and `changed_in`.

- `api_version`: is the current version of the Api. In our case we updated it to `#[api_version(2)]`.
- changed_in: is meant to describe for `decl_runtime_apis` macro past implementations of methods. In this case, we anotate our previous implementation with `#[changed_in(2)]`, telling the `decl_runtime_apis` macro that this is the implementation to use before version 2. In fact, this attribute will rename the method name for the trait in the client side to `METHOD_before_version_VERSION`, so `trace_transaction_before_version_2` in our example.

The un-anotated method is considered the default implemetation, and holds the current `trace_transaction` signature, with the new header argument and the empty result.

### Using a versioned runtime api from the client

To identify which version to use depending on the Api version of the height we want to access the runtime in, we will use the `api_version` method available on a runtime api instance:

```
let api_version = api
	.api_version::<dyn DebugRuntimeApi<B>>(&MY_BLOCK_ID)
	.map_err(|e| internal_err(format!("Runtime api access error: {:?}", e)))?
	.ok_or_else(|| {
		internal_err(format!(
			"Could not find `DebugRuntimeApi` at {:?}.",
			MY_BLOCK_ID
		))
	})?;

// ...
if api_version >= 2 {
  api.trace_transaction(&MY_BLOCK_ID, &MY_HEADER, ...)
} else {
  api.trace_transaction_before_version_2(&MY_BLOCK_ID, ...)
}
```

In the example above we updated the result type, that means we will need different logics to handle the response for each version in the client. This support needs to cover all versions added historically for any Runtime Api from genesis, as well as all have access to all the primitives that exist or existed in the runtime api methods' signatures.
