---
name: developing-rpcs
description: Develops and extends RPC methods for the Moonbeam node including Ethereum-compatible RPCs and custom Moonbeam RPCs. Use when adding RPC endpoints, implementing Ethereum RPC compatibility, debugging RPC issues, extending existing methods, or adding tracing/debug capabilities.
---

# RPC Development

## Contents
- [RPC Architecture](#rpc-architecture)
- [Adding a New RPC Method](#adding-a-new-rpc-method)
- [Ethereum RPC Compatibility](#ethereum-rpc-compatibility)
- [Debug and Trace RPCs](#debug-and-trace-rpcs)
- [EVM Tracing](#evm-tracing)
- [Testing RPCs](#testing-rpcs)
- [Common RPC Patterns](#common-rpc-patterns)

## RPC Architecture

### RPC Layer Structure

```
client/
├── rpc/
│   ├── debug/           # debug_* RPC methods
│   ├── dev/             # Development-only RPCs
│   ├── finality/        # Parachain finality RPCs
│   └── trace/           # trace_* RPC methods
├── rpc-core/
│   ├── debug/           # Debug RPC types
│   ├── trace/           # Trace RPC types
│   └── types/           # Shared RPC types
└── evm-tracing/         # EVM execution tracer
```

### RPC Categories

| Category | Prefix    | Purpose                               |
|----------|-----------|---------------------------------------|
| Ethereum | `eth_`    | Standard Ethereum RPCs (via Frontier) |
| Debug    | `debug_`  | Transaction debugging                 |
| Trace    | `trace_`  | Execution tracing                     |
| Web3     | `web3_`   | Web3 utilities                        |
| Net      | `net_`    | Network info                          |
| Moonbeam | `moon_`   | Custom Moonbeam methods               |
| Txpool   | `txpool_` | Transaction pool                      |

## Adding a New RPC Method

### 1. Define RPC Interface

```rust
// client/rpc-core/src/my-api/mod.rs
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

#[rpc(server)]
pub trait MyApi {
    /// Get some custom data
    #[method(name = "moon_getData")]
    fn get_data(&self, param: String) -> RpcResult<MyData>;

    /// Async method example
    #[method(name = "moon_getDataAsync")]
    async fn get_data_async(&self, param: String) -> RpcResult<MyData>;
}
```

### 2. Define Types

```rust
// client/rpc-core/src/my-api/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyData {
    pub value: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<String>,
}
```

### 3. Implement the RPC

```rust
// client/rpc/src/my-api/mod.rs
use jsonrpsee::core::RpcResult;
use moonbeam_rpc_core_my_api::{MyApi, MyApiServer, MyData};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

pub struct MyApiHandler<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> MyApiHandler<C, B> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, B> MyApiServer for MyApiHandler<C, B>
where
    B: BlockT,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + Send + Sync + 'static,
    C::Api: MyRuntimeApi<B>,
{
    fn get_data(&self, param: String) -> RpcResult<MyData> {
        let api = self.client.runtime_api();
        let best = self.client.info().best_hash;

        let result = api.get_data(best, param)
            .map_err(|e| internal_err(format!("Runtime error: {:?}", e)))?;

        Ok(result.into())
    }

    async fn get_data_async(&self, param: String) -> RpcResult<MyData> {
        // Async implementation
        tokio::task::spawn_blocking(move || {
            // Heavy computation
        })
        .await
        .map_err(|e| internal_err(format!("Task error: {:?}", e)))?
    }
}

fn internal_err<T: ToString>(message: T) -> jsonrpsee::types::ErrorObjectOwned {
    jsonrpsee::types::ErrorObject::owned(
        jsonrpsee::types::error::INTERNAL_ERROR_CODE,
        message.to_string(),
        None::<()>,
    )
}
```

### 4. Register in Node Service

```rust
// node/service/src/rpc.rs
use moonbeam_rpc_my_api::{MyApiHandler, MyApiServer};

pub fn create_full<C, P, BE, A>(
    deps: FullDeps<C, P, BE, A>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    // ... trait bounds
{
    let mut io = RpcModule::new(());

    // ... existing RPCs

    // Add custom RPC
    io.merge(MyApiHandler::new(client.clone()).into_rpc())?;

    Ok(io)
}
```

## Ethereum RPC Compatibility

### Standard Methods (via Frontier)

```rust
// Implemented by Frontier
eth_blockNumber
eth_getBalance
eth_getStorageAt
eth_getTransactionCount
eth_getBlockByHash
eth_getBlockByNumber
eth_getTransactionByHash
eth_getTransactionReceipt
eth_sendRawTransaction
eth_call
eth_estimateGas
eth_gasPrice
eth_getLogs
// ... and more
```

### Custom Ethereum Methods

```rust
// For methods not in Frontier, add to client/rpc/
impl EthApiServer for EthHandler {
    // Custom implementation
    fn my_custom_eth_method(&self) -> RpcResult<Value> {
        // Implementation
    }
}
```

## Debug and Trace RPCs

### Debug Namespace

```rust
// client/rpc/debug/src/lib.rs
impl DebugServer for DebugHandler {
    async fn trace_transaction(
        &self,
        transaction_hash: H256,
        params: Option<TraceParams>,
    ) -> RpcResult<Value> {
        // Replay transaction with tracing
        let tracer = match params.and_then(|p| p.tracer) {
            Some(TracerType::CallTracer) => Box::new(CallTracer::new()),
            Some(TracerType::Raw) => Box::new(RawTracer::new()),
            None => Box::new(DefaultTracer::new()),
        };

        let result = self.trace_transaction_internal(transaction_hash, tracer).await?;
        Ok(result)
    }
}
```

### Trace Namespace

```rust
// client/rpc/trace/src/lib.rs
impl TraceServer for TraceHandler {
    async fn trace_filter(
        &self,
        filter: TraceFilter,
    ) -> RpcResult<Vec<LocalizedTrace>> {
        // Filter traces by criteria
        let traces = self.get_filtered_traces(
            filter.from_block,
            filter.to_block,
            filter.from_address,
            filter.to_address,
        ).await?;

        Ok(traces)
    }
}
```

## EVM Tracing

### Tracing Configuration

```rust
// client/evm-tracing/src/lib.rs
pub struct TracerConfig {
    pub disable_storage: bool,
    pub disable_memory: bool,
    pub disable_stack: bool,
    pub tracer: Option<String>,
    pub timeout: Option<String>,
}
```

### Runtime Tracing API

```rust
// runtime/evm_tracer/src/lib.rs
pub fn trace_transaction<T: EthereumConfig>(
    transaction: &Transaction,
    config: TracerConfig,
) -> Result<TracerOutput, Error> {
    // Execute transaction with tracing enabled
    let mut listener = TracingEventListener::new(config);

    pallet_evm::Runner::<T>::call(
        // ... params
    )?;

    Ok(listener.into_output())
}
```

## Testing RPCs

### Unit Tests

```rust
#[tokio::test]
async fn test_get_data_rpc() {
    let client = create_test_client();
    let handler = MyApiHandler::new(client);

    let result = handler.get_data("test".to_string()).unwrap();

    assert_eq!(result.value, 42);
}
```

### Integration Tests

```typescript
// test/suites/dev/moonbase/test-rpc/test-rpc-custom.ts
describeSuite({
  id: "D030101",
  title: "Custom RPC Tests",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should return custom data",
      test: async () => {
        const result = await context.viem().request({
          method: "moon_getData",
          params: ["test"],
        });

        expect(result.value).toBe(42);
      },
    });
  },
});
```

### Manual Testing

```bash
# Test RPC with curl
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method":"moon_getData", "params":["test"]}' \
  http://localhost:9944

# List available methods
curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method":"rpc_methods"}' \
  http://localhost:9944
```

## Common RPC Patterns

### Error Handling

```rust
use jsonrpsee::types::{error::INTERNAL_ERROR_CODE, ErrorObjectOwned};

fn internal_err<T: ToString>(msg: T) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, msg.to_string(), None::<()>)
}

fn not_found_err() -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32000, "Resource not found", None::<()>)
}

fn invalid_params_err<T: ToString>(msg: T) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, msg.to_string(), None::<()>)
}
```

### Async with Blocking

```rust
async fn heavy_computation(&self, param: String) -> RpcResult<Data> {
    let client = self.client.clone();

    tokio::task::spawn_blocking(move || {
        // CPU-intensive work
        compute_data(client, param)
    })
    .await
    .map_err(|e| internal_err(e))?
}
```

### Subscription RPCs

```rust
#[rpc(server)]
pub trait SubscriptionApi {
    #[subscription(
        name = "moon_subscribeData" => "moon_data",
        unsubscribe = "moon_unsubscribeData",
        item = DataUpdate
    )]
    fn subscribe_data(&self);
}

impl SubscriptionApiServer for Handler {
    fn subscribe_data(&self, sink: SubscriptionSink) -> SubscriptionResult {
        let stream = self.data_stream();

        tokio::spawn(async move {
            sink.pipe_from_stream(stream).await;
        });

        Ok(())
    }
}
```

## Key Files

- RPC Core Types: `client/rpc-core/src/`
- RPC Implementations: `client/rpc/src/`
- Debug RPC: `client/rpc/debug/src/lib.rs`
- Trace RPC: `client/rpc/trace/src/lib.rs`
- EVM Tracer: `client/evm-tracing/src/`
- Node Service: `node/service/src/rpc.rs`
