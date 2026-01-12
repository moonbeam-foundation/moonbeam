---
name: debugging-moonbeam
description: Debugs issues in the Moonbeam parachain including runtime panics, EVM execution failures, XCM message delivery problems, and client-side errors. Use when encountering transaction failures, unexpected behavior, state inconsistencies, block production issues, or RPC errors.
---

# Debugging Moonbeam

## Contents
- [Lazy Loading (Fork Mode)](#lazy-loading-fork-mode)
- [Runtime Debugging](#runtime-debugging)
- [EVM Debugging](#evm-debugging)
- [XCM Debugging](#xcm-debugging)
- [Client/RPC Debugging](#clientrpc-debugging)
- [Block Production Debugging](#block-production-debugging)
- [Common Error Patterns](#common-error-patterns)
- [Investigation Tools](#investigation-tools)

## Lazy Loading (Fork Mode)

Lazy loading allows running a local Moonbeam node that fetches state on-demand from a live RPC endpoint. This is the most powerful tool for debugging production issues as it lets you replay transactions against real chain state.

### Building with Lazy Loading

```bash
# Build with lazy-loading feature enabled
cargo build --release --features lazy-loading
```

### Basic Usage

```bash
# Fork from Moonbeam mainnet at latest block
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
  --sealing 6000

# Fork from specific block
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
  --lazy-loading-block 0x1234...abcd \
  --sealing 6000

# Fork Moonriver
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonriver.moonbeam.network \
  --sealing 6000

# Fork Moonbase Alpha
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbase.moonbeam.network \
  --sealing 6000
```

### Advanced Options

```bash
# Use custom runtime (test new runtime against production state)
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
  --lazy-loading-runtime-override ./target/release/wbuild/moonbeam-runtime/moonbeam_runtime.wasm \
  --sealing 6000

# Apply state overrides (modify storage for testing)
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
  --lazy-loading-state-overrides ./state-overrides.json \
  --sealing 6000

# Adjust RPC request throttling (avoid rate limits)
./target/release/moonbeam \
  --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
  --lazy-loading-delay-between-requests 100 \
  --lazy-loading-max-retries-per-request 5 \
  --sealing 6000
```

### State Overrides File Format

```json
{
  "0x1234...": {
    "balance": "0x1000000000000000000",
    "nonce": "0x0",
    "code": "0x...",
    "storage": {
      "0x0": "0x1234"
    }
  }
}
```

### Debugging Workflow with Lazy Loading

1. **Reproduce a production issue**:
   ```bash
   # Fork at the block before the problematic transaction
   ./target/release/moonbeam \
     --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
     --lazy-loading-block 0xBLOCK_BEFORE_ISSUE \
     --ethapi=debug,trace \
     --sealing manual
   ```

2. **Replay the failing transaction**:
   ```javascript
   // Get original tx details from production
   const tx = await prodProvider.getTransaction(txHash);

   // Replay on forked node with tracing
   const trace = await localProvider.send('debug_traceCall', [{
     from: tx.from,
     to: tx.to,
     data: tx.data,
     value: tx.value,
     gas: tx.gas
   }, 'latest', { tracer: 'callTracer' }]);
   ```

3. **Test runtime fixes**:
   ```bash
   # Build fixed runtime
   cargo build --release -p moonbeam-runtime

   # Test against production state
   ./target/release/moonbeam \
     --lazy-loading-remote-rpc https://rpc.api.moonbeam.network \
     --lazy-loading-runtime-override ./target/release/wbuild/moonbeam-runtime/moonbeam_runtime.wasm \
     --sealing 6000
   ```

### Performance Considerations

- Initial requests may be slow (state is fetched on-demand)
- Use a reliable, non-rate-limited RPC endpoint
- Consider running your own archive node for heavy debugging
- Expect ~20x slower execution compared to local state

### Common Use Cases

| Use Case               | Configuration                                           |
|------------------------|---------------------------------------------------------|
| Debug failed tx        | Fork at block before tx, replay with tracing            |
| Test migration         | Use `--lazy-loading-runtime-override` with new runtime  |
| Simulate whale actions | Use `--lazy-loading-state-overrides` to modify balances |
| Test governance        | Override voting power via state overrides               |
| Debug precompile       | Fork + trace precompile calls                           |

## Debugging Workflows

### Runtime Debugging

1. **Identify the failing component**:
   - Check logs for `WARN` or `ERROR` messages
   - Look for panic messages with stack traces
   - Identify which pallet or module is involved

2. **Reproduce locally**:
   ```bash
   # Run dev node with verbose logging
   RUST_LOG=debug ./target/release/moonbeam --dev --alice --sealing 6000 --rpc-port 9944

   # Target specific module logging
   RUST_LOG=pallet_parachain_staking=trace ./target/release/moonbeam --dev
   ```

3. **Add debug logging** in the pallet:
   ```rust
   use frame_support::log;
   log::debug!(target: "pallet-name", "Debug info: {:?}", value);
   ```

4. **Check storage state**:
   - Use Polkadot.js Apps to inspect storage
   - Query via RPC: `state_getStorage`

### EVM Debugging

1. **Enable EVM tracing**:
   ```bash
   # Run with tracing enabled
   ./target/release/moonbeam --dev --ethapi=debug,trace
   ```

2. **Use debug_traceTransaction**:
   ```javascript
   const trace = await provider.send('debug_traceTransaction', [txHash, {}]);
   ```

3. **Check precompile calls**:
   - Precompile addresses are deterministic (0x0000...0800+)
   - Look for revert reasons in trace output
   - Verify input encoding matches expected ABI

4. **Common EVM issues**:
   - Gas estimation failures: Check precompile gas costs
   - Revert without reason: Look at precompile error handling
   - State differences: Compare with expected EVM state

### XCM Debugging

1. **Enable XCM logging**:
   ```bash
   RUST_LOG=xcm=trace ./target/release/moonbeam --dev
   ```

2. **Check XCM message structure**:
   - Verify multilocation encoding
   - Check weight limits
   - Verify asset representation (local vs foreign)

3. **Common XCM issues**:
   - `TooExpensive`: Insufficient weight/fee
   - `UntrustedReserveLocation`: Asset origin mismatch
   - `AssetNotFound`: Asset not registered

4. **Test XCM locally**:
   ```bash
   # Use zombienet for multi-chain testing
   zombienet spawn zombienet/moonbeam.toml
   ```

### Client/RPC Debugging

1. **Check RPC method availability**:
   ```bash
   curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method":"rpc_methods"}' http://localhost:9944
   ```

2. **Verify client version compatibility**:
   - Ensure client matches runtime version
   - Check spec_version in runtime

3. **Debug connection issues**:
   ```bash
   # Test WebSocket connection
   wscat -c ws://localhost:9944
   ```

### Block Production Debugging

1. **Check collator status**:
   - Verify author mapping: `AuthorMapping.MappingWithDeposit`
   - Check nimbus keys are registered

2. **Monitor block production**:
   ```bash
   # Watch block events
   RUST_LOG=cumulus=debug ./target/release/moonbeam
   ```

3. **Common block issues**:
   - Missed slots: Check collator selection
   - Invalid blocks: Check weight limits
   - Orphaned blocks: Check finality

## Key Log Targets

| Target            | Component                       |
|-------------------|---------------------------------|
| `pallet_evm`      | EVM execution                   |
| `pallet_ethereum` | Ethereum transaction processing |
| `xcm`             | XCM message handling            |
| `cumulus`         | Parachain consensus             |
| `moonbeam_rpc`    | Custom RPC methods              |
| `frontier`        | Ethereum compatibility layer    |

## Useful RPC Methods for Debugging

```javascript
// Get transaction receipt with logs
eth_getTransactionReceipt(txHash)

// Trace transaction execution
debug_traceTransaction(txHash, {tracer: 'callTracer'})

// Get storage at specific block
eth_getStorageAt(address, slot, blockNumber)

// Check pending transactions
txpool_content()

// Get block details
eth_getBlockByNumber(blockNumber, true)
```

## Test-Driven Debugging

1. **Write a failing test**:
   ```typescript
   // test/suites/dev/moonbase/test-debug/test-issue-xxxx.ts
   describeSuite({
     id: "Dxxxxxx",
     title: "Bug reproduction for issue #xxxx",
     foundationMethods: "dev",
     testCases: ({ context, it }) => {
       it({ id: "T01", title: "Reproduces the bug", test: async () => {
         // Reproduction steps
       }});
     }
   });
   ```

2. **Run the test**:
   ```bash
   cd test && pnpm moonwall test dev_moonbase Dxxxxxx
   ```

## Files to Check

- Runtime logs: Check `frame_support::log` outputs
- Precompile errors: `/precompiles/*/src/lib.rs` - look for `Err()` returns
- XCM barriers: `/runtime/*/xcm_config.rs` - check barrier implementations
- Weight limits: `/runtime/*/weights/` - verify weight calculations

## Common Error Patterns

### Dispatch Errors

| Error                 | Likely Cause         | Investigation                    |
|-----------------------|----------------------|----------------------------------|
| `BadOrigin`           | Wrong caller type    | Check origin requirements        |
| `InsufficientBalance` | Not enough funds     | Check free vs reserved balance   |
| `StorageOverflow`     | Arithmetic overflow  | Check bounded types              |
| `TooManyDelegations`  | Hit delegation limit | Check MaxDelegationsPerDelegator |

### EVM Errors

| Error          | Cause                       | Debug Steps                                    |
|----------------|-----------------------------|------------------------------------------------|
| `OutOfGas`     | Gas limit too low           | Increase gas, check precompile costs           |
| `Revert`       | Contract/precompile failure | Check revert reason, trace tx                  |
| `InvalidNonce` | Nonce mismatch              | Check pending txs, use eth_getTransactionCount |
| `IntrinsicGas` | Base gas cost not met       | Ensure gas >= 21000 + calldata                 |

### Precompile Errors

```rust
// Common revert patterns to search for
revert("Invalid input")
revert("Not enough balance")
revert("Permission denied")
Err(PrecompileFailure::Error { exit_status: ... })
```

## Investigation Tools

### Polkadot.js Apps

```
Developer → Chain State → Select pallet → Query storage
Developer → Extrinsics → Submit test calls
Developer → RPC Calls → Raw RPC queries
```

### Substrate Debug Tools

```bash
# Decode storage key
subkey inspect --public "0x1234..."

# Parse extrinsic
subxt explore --url ws://localhost:9944

# Metadata inspection
frame-omni-bencher v1 metadata --runtime path/to/runtime.wasm
```

## Reproducing Issues

### From Transaction Hash

```typescript
// Get tx details
const tx = await api.rpc.eth.getTransactionByHash(txHash);
const receipt = await api.rpc.eth.getTransactionReceipt(txHash);

// Replay on dev node
const rawTx = await context.createTxn!({
  to: tx.to,
  data: tx.input,
  value: tx.value,
  gas: tx.gas,
});
await context.createBlock(rawTx);
```

### From Block State

```bash
# Fork mainnet state with Chopsticks
npx @acala-network/chopsticks@latest \
  --config chopsticks/moonbeam.yml \
  --block 5000000
```

## Performance Debugging

### Identify Slow Extrinsics

```bash
# Run with benchmark feature
RUST_LOG=runtime::executive=trace ./target/release/moonbeam --dev
```

### Profile Weights

```rust
// Add weight logging
log::info!(
    target: "benchmark",
    "Extrinsic weight: reads={}, writes={}, compute={}",
    weight.proof_size(),
    weight.ref_time()
);
```

## Memory Debugging

```bash
# Check for memory leaks
RUST_BACKTRACE=1 cargo test --release -- --nocapture

# Profile memory usage
heaptrack ./target/release/moonbeam --dev
```

## Network Debugging

### Sync Issues

```bash
# Check peer connections
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_peers"}' \
  http://localhost:9944

# Check sync state
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_syncState"}' \
  http://localhost:9944
```

### Finality Issues

```bash
# Check GRANDPA state
RUST_LOG=grandpa=debug ./target/release/moonbeam

# Verify finalized block
curl -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"chain_getFinalizedHead"}' \
  http://localhost:9944
```

## Debugging Checklist

1. [ ] Identify the failing operation (extrinsic, RPC, block)
2. [ ] Check logs for error messages
3. [ ] Reproduce on local dev node
4. [ ] Enable relevant debug logging
5. [ ] Trace execution path
6. [ ] Identify root cause
7. [ ] Write failing test
8. [ ] Implement fix
9. [ ] Verify fix passes test
10. [ ] Check for regressions
