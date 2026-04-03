# Block Replay Regression Test

Replays a range of live-chain blocks through the locally compiled runtime
(via the lazy-loading node) and verifies that every transaction produces the
same output (events, gas, logs, status) as the original chain.

## Quick start

```bash
# 1. Build the moonbeam node and runtime
cargo build --release -p moonbeam -p moonbeam-runtime

# 2. Run the test (forks at the latest on-chain block automatically)
cd test
pnpm moonwall test block_replay_moonbeam

# Optional: control how many blocks to replay (default 10)
REPLAY_BLOCK_COUNT=20 pnpm moonwall test block_replay_moonbeam

# Optional: start from a specific block (must be after the fork point)
REPLAY_BLOCK=15073695 REPLAY_BLOCK_COUNT=5 pnpm moonwall test block_replay_moonbeam
```

## How it works

```
┌─────────────────────────────────────────────────────────────────┐
│  LIVE CHAIN (original data source)                              │
│  Fetches: Substrate blocks/events + Ethereum blocks/receipts    │
└────────────────────────┬────────────────────────────────────────┘
                         │ compare per-tx (matched by hash)
┌────────────────────────▼────────────────────────────────────────┐
│  LAZY-LOADING NODE (new runtime)                                │
│  Forks from the latest block, replays transactions, seals       │
└─────────────────────────────────────────────────────────────────┘
```

The node forks at the latest on-chain block when it starts. The test
auto-detects the fork point and replays the blocks that follow it.

### Per-block flow

1. **Fetch** the original block from the live chain (Substrate events +
   Ethereum receipts) via RPC.
2. **Extract** user transactions:
   - Ethereum: raw signed transaction bytes reconstructed from RPC tx objects
   - Substrate: raw extrinsic bytes (non-inherent, non-ethereum)
3. **Submit** extracted transactions to the local lazy-loading node (with
   retries for transient lazy-loading delays).
4. **Seal** a new block (`createBlock` via manual seal).
5. **Fetch** the newly produced block's events + receipts from the local node.
6. **Match** original and replayed receipts **by transaction hash** (handles
   tx pool reordering and dropped transactions).
7. **Compare** per-transaction and **classify** each mismatch.

### What is compared

| Layer     | Fields compared                                         |
|-----------|---------------------------------------------------------|
| Ethereum  | `status`, `gasUsed`, log count, log address/topics/data |
| Substrate | Event section/method/data per extrinsic index           |

### Mismatch classification

Not all differences are regressions. The test classifies each mismatch:

| Classification | Meaning                                                      | Action          |
|----------------|--------------------------------------------------------------|-----------------|
| `timestamp`    | Caused by `block.timestamp` / `block.number` offset          | Expected, pass  |
| `gas`          | `gasUsed` differs but tx status is the same                  | Warning, pass   |
| `regression`   | True behavioural change                                      | **Fail**        |

**Timestamp-related** mismatches are expected because the lazy-loading node
uses the current system time, not the original block's timestamp. This causes:

- **Deadline reverts** — DeFi transactions with `require(block.timestamp <= deadline)` revert with low gas (~35k).
- **Timestamp log diffs** — Contracts that log `block.timestamp` produce different values.
- **TWAP / accumulator diffs** — Values derived from `block.timestamp` (e.g. Uniswap V3 oracle accumulators) differ. These are detected when the tx has no structural changes (same status, same log count/addresses/topics).

**Gas warnings** are informational — `gasUsed` differences where status
matches. These may indicate runtime gas cost changes worth reviewing but
are not treated as failures.

### What is intentionally skipped

- **Inherent extrinsics** (timestamp, parachainSystem, authorInherent): the
  lazy-loading node produces its own inherent data.
- **Inherent-related events** (parachainStaking round changes, author mapping,
  etc.): differ because of different inherent data.
- **Block-level fields** (stateRoot, hash, parentHash): the replayed block has
  a different position in the chain.
- **Dropped transactions**: transactions that the lazy-loading tx pool fails
  to include are reported but not counted as regressions.

## Environment variables

| Variable               | Required | Default                              | Description                                      |
|------------------------|----------|--------------------------------------|--------------------------------------------------|
| `REPLAY_BLOCK`         | ❌        | auto (fork point + 1)                | First block number to replay (must be after the fork point) |
| `REPLAY_BLOCK_COUNT`   | ❌        | `10`                                 | Number of consecutive blocks to replay           |
| `REPLAY_FORK_URL`      | ❌        | `https://rpc.api.moonbeam.network`   | RPC endpoint of the live chain                   |
| `REPLAY_SKIP_SUBSTRATE`| ❌        | `false`                              | Set to `"true"` to skip Substrate event comparison |

## Output

- Console logs show per-block pass/fail with mismatch details.
- `tmp/replayResults.json` — full structured results for every block.
- Per-block summary shows counts by classification:
  ```
  ✅ Block #15073693 — PASSED (32 timestamp, 3 gas, 77 substrate — all expected)
     ⚡ GAS 0x5403d637e40f7a9b… | 40461 → 43261
  ```

## Limitations & future work

- **Timestamp offset**: the lazy-loading node uses the current wall-clock
  time, not the original block's timestamp. The classifier handles most
  cases (deadline reverts, timestamp logs, TWAP accumulators) but exotic
  timestamp-dependent logic may produce false positives.
- **Tx pool drops**: the lazy-loading node may fail to include some
  transactions (especially in dense blocks) due to state-fetching delays.
  Dropped transactions are reported but not compared.
- **Sequential replay**: blocks are replayed sequentially from the fork point.
  After the first divergence, subsequent blocks may cascade into further
  differences (state has diverged). This is by design — it tells you exactly
  which block first diverged.
- **Substrate extrinsic replay**: signed Substrate extrinsics may fail to
  replay if their mortality era has expired relative to the lazy-loading
  node's block numbering. Ethereum transactions don't have this issue.
- **Block range size**: replaying thousands of blocks is slow due to RPC
  fetching and sequential sealing. Start with small ranges.
- **Inherent data**: the lazy-loading node uses mock inherent data, so any
  logic that depends on relay-chain state will differ.
