# Block Replay Regression Test

Replays a range of live-chain blocks through the locally compiled runtime
(via the lazy-loading node) and verifies that every transaction produces the
same output (events, gas, logs, status) as the original chain.

## Quick start

```bash
# 1. Build the moonbeam node and runtime
cargo build --release -p moonbeam -p moonbeam-runtime

# 2. Set the block range to replay
export REPLAY_BLOCK=12960000        # first block to replay
export REPLAY_BLOCK_COUNT=10        # number of consecutive blocks
export REPLAY_FORK_URL=https://rpc.api.moonbeam.network

# 3. Run the test
cd test
pnpm moonwall test block_replay_moonbeam
```

## How it works

```
┌─────────────────────────────────────────────────────────────────┐
│  LIVE CHAIN (original data source)                              │
│  Fetches: Substrate blocks/events + Ethereum blocks/receipts    │
└────────────────────────┬────────────────────────────────────────┘
                         │ compare per-tx
┌────────────────────────▼────────────────────────────────────────┐
│  LAZY-LOADING NODE (new runtime)                                │
│  Forks from REPLAY_BLOCK-1, replays transactions, seals blocks  │
└─────────────────────────────────────────────────────────────────┘
```

### Per-block flow

1. **Fetch** the original block from the live chain (Substrate events +
   Ethereum receipts) via RPC.
2. **Extract** user transactions:
   - Ethereum: raw signed transaction bytes via `eth_getRawTransactionByHash`
   - Substrate: raw extrinsic bytes (non-inherent, non-ethereum)
3. **Submit** extracted transactions to the local lazy-loading node.
4. **Seal** a new block (`createBlock` via manual seal).
5. **Fetch** the newly produced block's events + receipts from the local node.
6. **Compare** per-transaction:
   - Ethereum: `status`, `gasUsed`, logs (`address`, `topics`, `data`)
   - Substrate: event types and data per extrinsic index

### What is compared

| Layer     | Fields compared                                         |
|-----------|---------------------------------------------------------|
| Ethereum  | `status`, `gasUsed`, log count, log address/topics/data |
| Substrate | Event section/method/data per extrinsic index           |

### What is intentionally skipped

- **Inherent extrinsics** (timestamp, parachainSystem, authorInherent): the
  lazy-loading node produces its own inherent data.
- **Inherent-related events** (parachainStaking round changes, author mapping,
  etc.): differ because of different inherent data.
- **Block-level fields** (stateRoot, hash, parentHash): the replayed block has
  a different position in the chain.

## Environment variables

| Variable               | Required | Default                              | Description                                      |
|------------------------|----------|--------------------------------------|--------------------------------------------------|
| `REPLAY_BLOCK`         | ✅        | —                                    | First block number to replay                     |
| `REPLAY_BLOCK_COUNT`   | ❌        | `1`                                  | Number of consecutive blocks to replay           |
| `REPLAY_FORK_URL`      | ❌        | `https://rpc.api.moonbeam.network`   | RPC endpoint of the live chain                   |
| `REPLAY_SKIP_SUBSTRATE`| ❌        | `false`                              | Set to `"true"` to skip Substrate event comparison |

## Output

- Console logs show per-block pass/fail with mismatch details.
- `tmp/replayResults.json` — full structured results for every block.
- `tmp/replayBlockConfig.json` — the resolved replay configuration.

## Limitations & future work

- **Sequential replay**: blocks are replayed sequentially from the fork point.
  After the first divergence, subsequent blocks may cascade into further
  differences (state has diverged). This is by design — it tells you exactly
  which block first diverged.
- **Substrate extrinsic replay**: signed Substrate extrinsics may fail to
  replay if their mortality era has expired relative to the lazy-loading
  node's block numbering. Ethereum transactions don't have this issue.
- **Block range size**: replaying thousands of blocks is slow due to RPC
  fetching and sequential sealing. Start with small ranges around the
  runtime upgrade boundary.
- **Inherent data**: the lazy-loading node uses mock inherent data, so any
  logic that depends on relay-chain state or exact timestamps will differ.
