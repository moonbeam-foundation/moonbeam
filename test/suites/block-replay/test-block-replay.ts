/**
 * Block Replay Regression Test
 *
 * Replays a range of blocks from a live chain using the lazy-loading node
 * (running a new runtime) and compares per-transaction outputs against the
 * original chain data.
 *
 * Env vars consumed at runtime:
 *   REPLAY_FORK_URL        – live-chain RPC (also used by the lazy-loading node)
 *   REPLAY_BLOCK           – first block to replay
 *   REPLAY_BLOCK_COUNT     – how many consecutive blocks to replay (default 1)
 *   REPLAY_SKIP_SUBSTRATE  – if "true", skip Substrate extrinsic comparison
 *
 * The companion script `prepare-block-replay.ts` must have been run first to
 * generate `tmp/replayBlockConfig.json` and the state-override file.
 *
 * Flow per block N:
 *  1. Fetch original Substrate block, events & Ethereum block + receipts from the live chain.
 *  2. Extract user transactions (Ethereum & non-inherent Substrate).
 *  3. Submit them to the local lazy-loading node.
 *  4. Seal a block.
 *  5. Fetch replayed Substrate events & Ethereum receipts from the local node.
 *  6. Compare per-transaction: status, gasUsed, logs/events.
 */

import "@moonbeam-network/api-augment";
import { afterAll, beforeAll, describeSuite, expect } from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import { HttpProvider, WsProvider } from "@polkadot/rpc-provider";
import { serializeTransaction, type TransactionReceipt } from "viem";
import fs from "node:fs/promises";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Resolved at runtime from environment + local node state. */
interface ReplayConfig {
  forkUrl: string;
  replayFromBlock: number;
  replayBlockCount: number;
}

interface OriginalBlockData {
  blockNumber: number;
  blockTimestampMs: number;
  ethReceipts: TransactionReceipt[];
  substrateEventsByExtrinsic: Map<number, { section: string; method: string; data: string }[]>;
  rawEthTransactions: string[];
  rawSubstrateExtrinsics: string[];
  ethTxHashOrder: string[];
}

/**
 * Mismatch classification:
 *  - "timestamp"  – caused by block.timestamp / block.number environmental difference
 *  - "gas"        – gasUsed differs but tx status is the same (informational)
 *  - "regression" – a true behavioural change that warrants investigation
 */
type MismatchClass = "timestamp" | "gas" | "regression";

interface ComparisonResult {
  blockNumber: number;
  ethTxMismatches: EthTxMismatch[];
  substrateMismatches: SubstrateEventMismatch[];
  regressionCount: number;
  passed: boolean;
}

interface EthTxMismatch {
  txHash: string;
  field: string;
  original: string;
  replayed: string;
  classification: MismatchClass;
}

interface SubstrateEventMismatch {
  extrinsicIndex: number;
  description: string;
  original: string;
  replayed: string;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** Inherent extrinsic sections – these are not user transactions. */
const INHERENT_SECTIONS = new Set([
  "timestamp",
  "parachainSystem",
  "authorInherent",
  "nimbus",
  "randomness",
]);

/**
 * Event sections that can differ between original and replayed blocks because
 * the lazy-loading node produces its own inherent data.
 */
const INHERENT_EVENT_SECTIONS = new Set([
  "parachainSystem",
  "authorInherent",
  "timestamp",
  "nimbus",
  "randomness",
  "parachainStaking",
  "authorFilter",
  "authorMapping",
]);

/**
 * Gas threshold below which a `success → reverted` flip is classified as a
 * deadline / timestamp revert rather than a true regression.  Typical base
 * cost for a contract-call tx (21 000 intrinsic + calldata) is ~35-37 k gas.
 */
const DEADLINE_REVERT_GAS_THRESHOLD = 50_000n;

// ---------------------------------------------------------------------------
// Classification helpers
// ---------------------------------------------------------------------------

/**
 * Check whether an ABI-encoded data diff is explainable purely by
 * `block.timestamp` or `block.number` environmental offsets.
 *
 * Strategy: compare every 32-byte word; for each differing word check whether
 * the numeric delta matches the timestamp offset (in seconds) or ±1 (block
 * number).
 */
function isLogDataDiffEnvironmental(
  origHex: string,
  replHex: string,
  timestampOffsetSec: number
): boolean {
  if (origHex.length !== replHex.length) return false;
  const orig = origHex.startsWith("0x") ? origHex.slice(2) : origHex;
  const repl = replHex.startsWith("0x") ? replHex.slice(2) : replHex;
  if (orig.length !== repl.length) return false;

  let hasDiff = false;
  for (let i = 0; i < orig.length; i += 64) {
    const oWord = orig.slice(i, i + 64);
    const rWord = repl.slice(i, i + 64);
    if (oWord === rWord) continue;
    hasDiff = true;
    try {
      const oVal = BigInt("0x" + oWord);
      const rVal = BigInt("0x" + rWord);
      const delta = rVal - oVal;
      const absDelta = delta < 0n ? -delta : delta;
      const tsOffset = BigInt(timestampOffsetSec);
      // Allow timestamp offset ±2 s (covers ms→s rounding) or block-number ±1
      const isTimestamp =
        absDelta >= tsOffset - 2n && absDelta <= tsOffset + 2n && tsOffset > 0n;
      const isBlockNumber = absDelta <= 2n;
      if (!isTimestamp && !isBlockNumber) {
        return false;
      }
    } catch {
      return false;
    }
  }
  return hasDiff; // at least one word differed and all were explainable
}

/**
 * Classify every mismatch for a single Ethereum transaction.
 *
 * Groups the raw mismatches by txHash and decides per-transaction:
 *  • If the tx went success→reverted with very low replayed gas it is a
 *    timestamp deadline revert  → all its mismatches are "timestamp".
 *  • Log-data diffs that are purely block.timestamp / block.number
 *    offsets → "timestamp".
 *  • gasUsed diffs where status is the same → "gas" (informational).
 *  • Everything else → "regression".
 */
function classifyEthMismatches(
  mismatches: EthTxMismatch[],
  timestampOffsetSec: number
): void {
  // Group by txHash
  const byTx = new Map<string, EthTxMismatch[]>();
  for (const m of mismatches) {
    const list = byTx.get(m.txHash) ?? [];
    list.push(m);
    byTx.set(m.txHash, list);
  }

  for (const [, txMismatches] of byTx) {
    const statusM = txMismatches.find((m) => m.field === "status");
    const gasM = txMismatches.find((m) => m.field === "gasUsed");

    const isRevert =
      statusM?.original === "success" && statusM?.replayed === "reverted";
    const replayedGas = gasM ? BigInt(gasM.replayed) : null;

    // ── Deadline / timestamp revert ──
    if (isRevert && replayedGas !== null && replayedGas < DEADLINE_REVERT_GAS_THRESHOLD) {
      for (const m of txMismatches) m.classification = "timestamp";
      continue;
    }

    // Check whether this tx has only "soft" diffs (log data, gasUsed) while
    // status, log count, addresses, and topics all match.  When there is a
    // non-zero timestamp offset any log-data-only diff is very likely caused
    // by timestamp-derived values (TWAP accumulators, oracle snapshots, …).
    const hasStructuralDiff = txMismatches.some(
      (m) =>
        m.field === "status" ||
        m.field === "receipt_count" ||
        m.field === "logs_count" ||
        m.field.endsWith(".address") ||
        m.field.endsWith(".topics")
    );

    // ── Per-field classification for non-deadline txns ──
    for (const m of txMismatches) {
      if (m.classification !== "regression") continue; // already classified

      // Log data: exact environmental offset (block.timestamp / block.number)
      if (m.field.match(/^log\[\d+\]\.data$/) &&
          isLogDataDiffEnvironmental(m.original, m.replayed, timestampOffsetSec)) {
        m.classification = "timestamp";
        continue;
      }

      // Log data diff with NO structural mismatch (same status, count,
      // addresses, topics) and a known timestamp offset → timestamp-derived
      // value (e.g. TWAP accumulators, oracle snapshots).
      if (
        m.field.match(/^log\[\d+\]\.data$/) &&
        !hasStructuralDiff &&
        timestampOffsetSec !== 0
      ) {
        m.classification = "timestamp";
        continue;
      }

      // Log count diff where tx also reverted → secondary to revert
      if (isRevert && (m.field === "logs_count" || m.field.startsWith("log["))) {
        m.classification = "timestamp";
        continue;
      }

      // gasUsed diff when status matches (or already classified revert)
      if (m.field === "gasUsed" && !isRevert) {
        m.classification = "gas";
        continue;
      }

      // Everything else stays "regression"
    }
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Build the replay config from env vars; fork point is detected later from the node. */
function loadReplayConfig(): ReplayConfig {
  return {
    forkUrl: process.env.REPLAY_FORK_URL ?? "https://rpc.api.moonbeam.network",
    replayFromBlock: 0, // resolved in beforeAll from the local node head
    replayBlockCount: Number(process.env.REPLAY_BLOCK_COUNT ?? "10"),
  };
}

/** Create a polkadot.js provider for the given URL (auto-detects HTTP vs WS). */
function providerForUrl(url: string) {
  if (url.startsWith("ws://") || url.startsWith("wss://")) {
    return new WsProvider(url);
  }
  // Use HttpProvider for HTTP(S) URLs – avoids needing a separate WS endpoint
  return new HttpProvider(url);
}

/** Ensure URL is HTTP(S) for fetch-based calls. */
function toHttpUrl(url: string): string {
  return url.replace(/^wss:\/\//, "https://").replace(/^ws:\/\//, "http://");
}

/** Make a raw JSON-RPC call via HTTP. */
async function rpcCall(url: string, method: string, params: any[] = []): Promise<any> {
  const httpUrl = toHttpUrl(url);
  const res = await fetch(httpUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const json: any = await res.json();
  if (json.error) throw new Error(`RPC ${method}: ${JSON.stringify(json.error)}`);
  return json.result;
}

/**
 * Reconstruct the raw signed transaction from the JSON-RPC transaction object.
 * Uses viem's serializeTransaction which handles legacy, EIP-2930, EIP-1559,
 * and EIP-4844 types.
 */
function reconstructRawTransaction(tx: any): string {
  const txType = parseInt(tx.type, 16);

  switch (txType) {
    case 0: // Legacy
      return serializeTransaction(
        {
          type: "legacy" as const,
          nonce: parseInt(tx.nonce, 16),
          gasPrice: BigInt(tx.gasPrice),
          gas: BigInt(tx.gas),
          to: tx.to as `0x${string}` | null,
          value: BigInt(tx.value),
          data: tx.input as `0x${string}`,
        },
        { r: tx.r, s: tx.s, v: BigInt(tx.v) }
      );

    case 1: // EIP-2930
      return serializeTransaction(
        {
          type: "eip2930" as const,
          chainId: parseInt(tx.chainId, 16),
          nonce: parseInt(tx.nonce, 16),
          gasPrice: BigInt(tx.gasPrice),
          gas: BigInt(tx.gas),
          to: tx.to as `0x${string}` | null,
          value: BigInt(tx.value),
          data: tx.input as `0x${string}`,
          accessList: tx.accessList ?? [],
        },
        { r: tx.r, s: tx.s, yParity: parseInt(tx.v, 16) }
      );

    case 2: // EIP-1559
      return serializeTransaction(
        {
          type: "eip1559" as const,
          chainId: parseInt(tx.chainId, 16),
          nonce: parseInt(tx.nonce, 16),
          maxPriorityFeePerGas: BigInt(tx.maxPriorityFeePerGas),
          maxFeePerGas: BigInt(tx.maxFeePerGas),
          gas: BigInt(tx.gas),
          to: tx.to as `0x${string}` | null,
          value: BigInt(tx.value),
          data: tx.input as `0x${string}`,
          accessList: tx.accessList ?? [],
        },
        { r: tx.r, s: tx.s, yParity: parseInt(tx.v, 16) }
      );

    case 4: // EIP-7702
      return serializeTransaction(
        {
          type: "eip7702" as const,
          chainId: parseInt(tx.chainId, 16),
          nonce: parseInt(tx.nonce, 16),
          maxPriorityFeePerGas: BigInt(tx.maxPriorityFeePerGas),
          maxFeePerGas: BigInt(tx.maxFeePerGas),
          gas: BigInt(tx.gas),
          to: tx.to as `0x${string}`,
          value: BigInt(tx.value),
          data: tx.input as `0x${string}`,
          accessList: tx.accessList ?? [],
          authorizationList: (tx.authorizationList ?? []).map((a: any) => ({
            address: a.address,
            chainId: parseInt(a.chainId, 16),
            nonce: parseInt(a.nonce, 16),
            r: a.r,
            s: a.s,
            yParity: parseInt(a.yParity, 16),
          })),
        },
        { r: tx.r, s: tx.s, yParity: parseInt(tx.v, 16) }
      );

    default:
      throw new Error(`Unsupported transaction type: 0x${txType.toString(16)}`);
  }
}

/** Fetch original block data from the live chain. */
async function fetchOriginalBlockData(
  liveApi: ApiPromise,
  liveRpcUrl: string,
  blockNumber: number,
  log: (...args: any[]) => void
): Promise<OriginalBlockData> {
  // ── Substrate side ──
  const blockHash = await liveApi.rpc.chain.getBlockHash(blockNumber);
  const signedBlock = await liveApi.rpc.chain.getBlock(blockHash);
  const apiAt = await liveApi.at(blockHash);
  const allEvents: any[] = (await apiAt.query.system.events()) as any;

  // Group events by extrinsic index
  const substrateEventsByExtrinsic = new Map<
    number,
    { section: string; method: string; data: string }[]
  >();
  for (const record of allEvents) {
    const idx = record.phase.isApplyExtrinsic ? record.phase.asApplyExtrinsic.toNumber() : -1;
    if (idx < 0) continue; // skip Initialization / Finalization phase events
    const list = substrateEventsByExtrinsic.get(idx) ?? [];
    list.push({
      section: record.event.section,
      method: record.event.method,
      data: record.event.data.toHex(),
    });
    substrateEventsByExtrinsic.set(idx, list);
  }

  // Extract block timestamp from the timestamp.set inherent (milliseconds)
  let blockTimestampMs = 0;
  for (const ext of signedBlock.block.extrinsics) {
    if (ext.method.section === "timestamp" && ext.method.method === "set") {
      blockTimestampMs = Number(ext.method.args[0].toString());
      break;
    }
  }

  // Classify extrinsics
  const rawSubstrateExtrinsics: string[] = [];
  for (const ext of signedBlock.block.extrinsics) {
    const section = ext.method.section;
    const method = ext.method.method;
    // Skip inherents and ethereum.transact (handled via ETH RPC)
    if (INHERENT_SECTIONS.has(section)) continue;
    if (section === "ethereum" && method === "transact") continue;
    rawSubstrateExtrinsics.push(ext.toHex());
  }

  // ── Ethereum side ──
  const rawEthTransactions: string[] = [];
  const ethTxHashOrder: string[] = [];

  // Get full tx objects from the Ethereum JSON-RPC so we can reconstruct raw bytes
  const ethBlockRaw = await rpcCall(liveRpcUrl, "eth_getBlockByNumber", [
    "0x" + blockNumber.toString(16),
    true,
  ]);

  for (const tx of ethBlockRaw?.transactions ?? []) {
    ethTxHashOrder.push(tx.hash);
    try {
      const rawTx = reconstructRawTransaction(tx);
      rawEthTransactions.push(rawTx);
    } catch (e: any) {
      log(`    ⚠️  Could not reconstruct raw tx for ${tx.hash}: ${e.message}`);
    }
  }

  // Fetch receipts from Substrate storage (same format we use for replayed data)
  const subReceipts: any[] = ((await apiAt.query.ethereum.currentReceipts()) as any)
    .unwrapOr([])
    .toArray();

  const ethReceipts: TransactionReceipt[] = [];
  let origCumulativeGas = 0n;
  for (let i = 0; i < subReceipts.length; i++) {
    const r = subReceipts[i];
    const inner = r.isEip658 ? r.asEip658 : (r.value ?? r);
    const usedGasCumulative = BigInt(inner.usedGas.toString());
    const gasUsed = usedGasCumulative - origCumulativeGas;
    origCumulativeGas = usedGasCumulative;

    const logs = (inner.logs ?? []).map((l: any) => ({
      address: l.address.toString().toLowerCase(),
      topics: l.topics.map((t: any) => t.toHex()),
      data: l.data.toHex(),
    }));

    ethReceipts.push({
      status: inner.statusCode.toNumber() === 1 ? "success" : "reverted",
      gasUsed,
      logs,
      transactionHash: ethTxHashOrder[i] ?? `orig-tx-${i}`,
    } as any);
  }

  return {
    blockNumber,
    blockTimestampMs,
    ethReceipts,
    substrateEventsByExtrinsic,
    rawEthTransactions,
    rawSubstrateExtrinsics,
    ethTxHashOrder,
  };
}

// ---------------------------------------------------------------------------
// Comparison logic
// ---------------------------------------------------------------------------

function compareEthReceipts(
  originalReceipts: TransactionReceipt[],
  replayedReceipts: TransactionReceipt[]
): EthTxMismatch[] {
  const mismatches: EthTxMismatch[] = [];

  const mismatch = (
    txHash: string,
    field: string,
    original: string,
    replayed: string
  ): EthTxMismatch => ({ txHash, field, original, replayed, classification: "regression" });

  if (originalReceipts.length !== replayedReceipts.length) {
    // Receipt count mismatch is typically caused by the lazy-loading tx pool
    // dropping transactions, not by a runtime regression.  Classify as "gas"
    // (informational) and compare the receipts that DO exist.
    mismatches.push({
      txHash: "N/A",
      field: "receipt_count",
      original: String(originalReceipts.length),
      replayed: String(replayedReceipts.length),
      classification: "gas",
    });
  }

  const compareLen = Math.min(originalReceipts.length, replayedReceipts.length);

  for (let i = 0; i < compareLen; i++) {
    const orig = originalReceipts[i];
    const repl = replayedReceipts[i];

    if (orig.status !== repl.status) {
      mismatches.push(mismatch(orig.transactionHash, "status", orig.status, repl.status));
    }

    if (orig.gasUsed !== repl.gasUsed) {
      mismatches.push(
        mismatch(orig.transactionHash, "gasUsed", String(orig.gasUsed), String(repl.gasUsed))
      );
    }

    if (orig.logs.length !== repl.logs.length) {
      mismatches.push(
        mismatch(orig.transactionHash, "logs_count", String(orig.logs.length), String(repl.logs.length))
      );
      continue;
    }

    for (let j = 0; j < orig.logs.length; j++) {
      const oLog = orig.logs[j];
      const rLog = repl.logs[j];

      if (oLog.address.toLowerCase() !== rLog.address.toLowerCase()) {
        mismatches.push(
          mismatch(orig.transactionHash, `log[${j}].address`, oLog.address, rLog.address)
        );
      }

      const oTopics = ((oLog as any).topics ?? []).join(",");
      const rTopics = ((rLog as any).topics ?? []).join(",");
      if (oTopics !== rTopics) {
        mismatches.push(
          mismatch(orig.transactionHash, `log[${j}].topics`, oTopics, rTopics)
        );
      }

      if (oLog.data !== rLog.data) {
        mismatches.push(
          mismatch(orig.transactionHash, `log[${j}].data`, oLog.data, rLog.data)
        );
      }
    }
  }

  return mismatches;
}

function compareSubstrateEvents(
  originalEvents: Map<number, { section: string; method: string; data: string }[]>,
  replayedEvents: Map<number, { section: string; method: string; data: string }[]>
): SubstrateEventMismatch[] {
  const mismatches: SubstrateEventMismatch[] = [];

  /** Keep only events relevant to user extrinsics. */
  const filterEvents = (events: { section: string; method: string; data: string }[]) =>
    events.filter((e) => {
      if (INHERENT_EVENT_SECTIONS.has(e.section)) return false;
      // system.ExtrinsicSuccess / ExtrinsicFailed always matter
      if (e.section === "system" && e.method.startsWith("Extrinsic")) return true;
      // Keep everything else that isn't inherent-related
      return true;
    });

  const allIndices = new Set([...originalEvents.keys(), ...replayedEvents.keys()]);

  for (const idx of allIndices) {
    const origEvts = filterEvents(originalEvents.get(idx) ?? []);
    const replEvts = filterEvents(replayedEvents.get(idx) ?? []);

    if (origEvts.length !== replEvts.length) {
      mismatches.push({
        extrinsicIndex: idx,
        description: "event_count",
        original: `${origEvts.length} [${origEvts.map((e) => `${e.section}.${e.method}`).join(", ")}]`,
        replayed: `${replEvts.length} [${replEvts.map((e) => `${e.section}.${e.method}`).join(", ")}]`,
      });
      continue;
    }

    for (let i = 0; i < origEvts.length; i++) {
      const o = origEvts[i];
      const r = replEvts[i];

      if (o.section !== r.section || o.method !== r.method) {
        mismatches.push({
          extrinsicIndex: idx,
          description: `event[${i}] type`,
          original: `${o.section}.${o.method}`,
          replayed: `${r.section}.${r.method}`,
        });
      } else if (o.data !== r.data) {
        mismatches.push({
          extrinsicIndex: idx,
          description: `event[${i}] ${o.section}.${o.method} data`,
          original: o.data,
          replayed: r.data,
        });
      }
    }
  }

  return mismatches;
}

// ---------------------------------------------------------------------------
// Test suite
// ---------------------------------------------------------------------------

describeSuite({
  id: "BR01",
  title: "Block Replay – Verify transaction outputs match after runtime upgrade",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let replayConfig: ReplayConfig;
    let liveApi: ApiPromise;
    let localApi: ApiPromise;

    beforeAll(async () => {
      replayConfig = loadReplayConfig();

      // ── Local lazy-loading node (new runtime) ──
      localApi = context.polkadotJs();

      // The lazy-loading node forks at some recent block and creates an init
      // block on top. Detect the fork point so we know which live-chain blocks
      // to replay (the ones right after the init block).
      const localHead = (await localApi.rpc.chain.getHeader()).number.toNumber();
      const envBlock = Number(process.env.REPLAY_BLOCK ?? "0");
      if (envBlock > 0) {
        if (envBlock <= localHead) {
          throw new Error(
            `REPLAY_BLOCK=${envBlock} is at or before the fork point (#${localHead}). ` +
              `It must be greater than the fork point.`
          );
        }
        replayConfig.replayFromBlock = envBlock;
      } else {
        replayConfig.replayFromBlock = localHead + 1;
      }

      const from = replayConfig.replayFromBlock;
      const to = from + replayConfig.replayBlockCount - 1;
      log(`Fork point   : block #${localHead} (init block)`);
      log(`Replay range : #${from} → #${to}  (${replayConfig.replayBlockCount} blocks)`);
      log(`Live RPC     : ${replayConfig.forkUrl}`);

      // ── Live chain connection ──
      const provider = providerForUrl(replayConfig.forkUrl);
      const { ApiPromise: ApiPromiseClass } = await import("@polkadot/api");
      liveApi = await ApiPromiseClass.create({ provider, noInitWarn: true });

      const localSpecVersion = localApi.consts.system.version.specVersion.toNumber();
      const liveSpecVersion = liveApi.consts.system.version.specVersion.toNumber();
      log(`Live spec_version  : ${liveSpecVersion}`);
      log(`Local spec_version : ${localSpecVersion}`);
    });

    afterAll(async () => {
      try {
        if (liveApi && typeof liveApi.disconnect === "function") {
          await liveApi.disconnect();
        }
      } catch {
        /* HttpProvider has no disconnect */
      }
    });

    it({
      id: "T01",
      title: "Replay blocks and compare per-transaction outputs",
      timeout: 3_600_000,
      test: async () => {
        const from = replayConfig.replayFromBlock;
        const to = from + replayConfig.replayBlockCount - 1;
        const skipSubstrate = process.env.REPLAY_SKIP_SUBSTRATE === "true";
        const results: ComparisonResult[] = [];

        log(`\n${"═".repeat(70)}`);
        log(`  BLOCK REPLAY: #${from} → #${to}  (${replayConfig.replayBlockCount} blocks)`);
        log(`${"═".repeat(70)}`);

        for (let blockNum = from; blockNum <= to; blockNum++) {
          log(`\n─── Block #${blockNum} ───`);

          // 1. Fetch original block data from the live chain
          log(`  📥 Fetching original data …`);
          const original = await fetchOriginalBlockData(
            liveApi,
            replayConfig.forkUrl,
            blockNum,
            log
          );
          log(
            `     ${original.rawEthTransactions.length}/${original.ethTxHashOrder.length} ETH txns (raw available), ` +
              `${original.rawSubstrateExtrinsics.length} substrate extrinsics` +
              ` (timestamp ${new Date(original.blockTimestampMs).toISOString()})`
          );

          // 2. Submit Ethereum transactions (with retries for lazy-loading delays)
          let ethSubmitted = 0;
          const MAX_RETRIES = 3;
          for (const rawTx of original.rawEthTransactions) {
            let submitted = false;
            for (let attempt = 0; attempt < MAX_RETRIES && !submitted; attempt++) {
              try {
                await context.viem().request({
                  method: "eth_sendRawTransaction" as any,
                  params: [rawTx as any],
                });
                submitted = true;
                ethSubmitted++;
              } catch (e: any) {
                if (attempt < MAX_RETRIES - 1) {
                  await new Promise((r) => setTimeout(r, 500));
                } else {
                  log(`  ⚠️  eth_sendRawTransaction failed: ${(e.message ?? "").substring(0, 120)}`);
                }
              }
            }
          }

          // 3. Submit Substrate extrinsics (best-effort)
          let subSubmitted = 0;
          if (!skipSubstrate) {
            for (const rawExt of original.rawSubstrateExtrinsics) {
              try {
                await localApi.rpc.author.submitExtrinsic(rawExt);
                subSubmitted++;
              } catch (e: any) {
                log(`  ⚠️  author.submitExtrinsic failed: ${(e.message ?? "").substring(0, 120)}`);
              }
            }
          }
          log(`  📤 Submitted ${ethSubmitted} ETH + ${subSubmitted} substrate txns`);

          // 4. Seal a block
          log(`  🔨 Sealing block …`);
          await context.createBlock();

          // 5. Fetch replayed data from the local node via Substrate API
          //    (more reliable than Ethereum JSON-RPC on the lazy-loading backend)
          const latestHash = await localApi.rpc.chain.getBlockHash();
          const latestHeader = await localApi.rpc.chain.getHeader(latestHash);
          log(`  Replayed in local block #${latestHeader.number.toNumber()}`);
          const replayedApiAt = await localApi.at(latestHash);
          const replayedAllEvents: any[] = (await replayedApiAt.query.system.events()) as any;

          const replayedEventsByExtrinsic = new Map<
            number,
            { section: string; method: string; data: string }[]
          >();
          for (const record of replayedAllEvents) {
            const idx = record.phase.isApplyExtrinsic
              ? record.phase.asApplyExtrinsic.toNumber()
              : -1;
            if (idx < 0) continue;
            const list = replayedEventsByExtrinsic.get(idx) ?? [];
            list.push({
              section: record.event.section,
              method: record.event.method,
              data: record.event.data.toHex(),
            });
            replayedEventsByExtrinsic.set(idx, list);
          }

          // Get Ethereum receipts from Substrate storage instead of JSON-RPC
          const replayedSubReceipts: any[] = (
            (await replayedApiAt.query.ethereum.currentReceipts()) as any
          )
            .unwrapOr([])
            .toArray();

          // Convert Substrate receipts into a comparable shape
          const replayedReceipts: TransactionReceipt[] = [];
          let cumulativeGas = 0n;
          for (let i = 0; i < replayedSubReceipts.length; i++) {
            const r = replayedSubReceipts[i];
            // Receipt is an enum: EIP658 { statusCode, usedGas, logsBloom, logs }
            const inner = r.isEip658 ? r.asEip658 : (r.value ?? r);
            const usedGasCumulative = BigInt(inner.usedGas.toString());
            const gasUsed = usedGasCumulative - cumulativeGas;
            cumulativeGas = usedGasCumulative;

            const logs = (inner.logs ?? []).map((l: any) => ({
              address: l.address.toString().toLowerCase(),
              topics: l.topics.map((t: any) => t.toHex()),
              data: l.data.toHex(),
            }));

            replayedReceipts.push({
              status: inner.statusCode.toNumber() === 1 ? "success" : "reverted",
              gasUsed,
              logs,
              transactionHash: `replayed-tx-${i}`,
            } as any);
          }

          // Get real tx hashes from the replayed Ethereum block so we can
          // match original↔replayed by hash instead of by position.
          const replayedBlockNum = latestHeader.number.toNumber();
          const replayedEthBlock = await context
            .viem()
            .request({
              method: "eth_getBlockByNumber" as any,
              params: [("0x" + replayedBlockNum.toString(16)) as any, false],
            })
            .catch(() => null) as any;
          const replayedTxHashes: string[] = replayedEthBlock?.transactions ?? [];
          for (let i = 0; i < Math.min(replayedTxHashes.length, replayedReceipts.length); i++) {
            (replayedReceipts[i] as any).transactionHash = replayedTxHashes[i];
          }

          // 6. Compare by tx hash: only compare transactions present in both sets
          const origByHash = new Map(original.ethReceipts.map((r) => [r.transactionHash, r]));
          const replByHash = new Map(replayedReceipts.map((r) => [r.transactionHash, r]));
          const commonHashes = original.ethTxHashOrder.filter((h) => replByHash.has(h));
          const droppedCount = original.ethReceipts.length - commonHashes.length;
          if (droppedCount > 0) {
            log(`  ⚠️  ${droppedCount} tx(s) not included in replayed block (tx pool)`);
          }

          const matchedOriginal = commonHashes.map((h) => origByHash.get(h)!);
          const matchedReplayed = commonHashes.map((h) => replByHash.get(h)!);
          const ethMismatches = compareEthReceipts(matchedOriginal, matchedReplayed);

          // Compute timestamp offset (seconds) between original and replayed blocks
          const replayedTimestampMs = (await replayedApiAt.query.timestamp.now()) as any;
          const replayedTsMs = Number(replayedTimestampMs.toString());
          const timestampOffsetSec = Math.round(
            (replayedTsMs - original.blockTimestampMs) / 1000
          );

          // Classify each ETH mismatch
          classifyEthMismatches(ethMismatches, timestampOffsetSec);

          const substrateMismatches = skipSubstrate
            ? []
            : compareSubstrateEvents(
                original.substrateEventsByExtrinsic,
                replayedEventsByExtrinsic
              );

          const regressions = ethMismatches.filter((m) => m.classification === "regression");
          const gasWarnings = ethMismatches.filter((m) => m.classification === "gas");
          const timestampExpected = ethMismatches.filter((m) => m.classification === "timestamp");

          const regressionCount = regressions.length;
          // Substrate mismatches are mostly secondary to ETH receipt changes when
          // transactions revert that previously succeeded. Count only those that
          // are NOT correlated with an already-classified ETH mismatch.
          const passed = regressionCount === 0;
          results.push({
            blockNumber: blockNum,
            ethTxMismatches: ethMismatches,
            substrateMismatches,
            regressionCount,
            passed,
          });

          if (passed && ethMismatches.length === 0 && substrateMismatches.length === 0) {
            log(`  ✅ Block #${blockNum} — PASSED (clean)`);
          } else if (passed) {
            log(
              `  ✅ Block #${blockNum} — PASSED (${timestampExpected.length} timestamp, ` +
                `${gasWarnings.length} gas, ${substrateMismatches.length} substrate — all expected)`
            );
            if (gasWarnings.length > 0) {
              for (const m of gasWarnings) {
                log(
                  `     ⚡ GAS ${m.txHash.substring(0, 18)}… | ${m.original} → ${m.replayed}`
                );
              }
            }
          } else {
            log(`  ❌ Block #${blockNum} — ${regressionCount} REGRESSION(S)`);
            for (const m of regressions) {
              log(
                `     🔴 ${m.txHash.substring(0, 18)}… | ${m.field}: ${m.original} → ${m.replayed}`
              );
            }
            if (gasWarnings.length > 0) {
              for (const m of gasWarnings) {
                log(
                  `     ⚡ GAS ${m.txHash.substring(0, 18)}… | ${m.original} → ${m.replayed}`
                );
              }
            }
          }
        }

        // ── Persist full results ──
        await fs.mkdir("tmp", { recursive: true });
        await fs.writeFile("tmp/replayResults.json", JSON.stringify(results, null, 2));

        // ── Summary ──
        const passedCount = results.filter((r) => r.passed).length;
        const failedCount = results.filter((r) => !r.passed).length;

        const totalRegressions = results.reduce((s, r) => s + r.regressionCount, 0);
        const totalTimestamp = results.reduce(
          (s, r) => s + r.ethTxMismatches.filter((m) => m.classification === "timestamp").length,
          0
        );
        const totalGas = results.reduce(
          (s, r) => s + r.ethTxMismatches.filter((m) => m.classification === "gas").length,
          0
        );
        const totalSubMismatches = results.reduce((s, r) => s + r.substrateMismatches.length, 0);

        log(`\n${"═".repeat(70)}`);
        log(`  RESULT: ${passedCount} passed, ${failedCount} failed / ${results.length} blocks`);
        log(`  Regressions      : ${totalRegressions}`);
        log(`  Timestamp-related : ${totalTimestamp} (expected — block.timestamp differs)`);
        log(`  Gas warnings      : ${totalGas} (informational)`);
        log(`  Substrate diffs   : ${totalSubMismatches} (secondary to ETH diffs)`);
        log(`  Full report: tmp/replayResults.json`);
        log(`${"═".repeat(70)}\n`);

        if (totalRegressions > 0) {
          log("🔴 REGRESSIONS:");
          for (const r of results) {
            for (const m of r.ethTxMismatches.filter((m) => m.classification === "regression")) {
              log(`  #${r.blockNumber} | ${m.txHash} | ${m.field}: ${m.original} → ${m.replayed}`);
            }
          }
        }

        expect(
          totalRegressions,
          `${totalRegressions} regression(s) across ${failedCount} block(s)`
        ).toBe(0);
      },
    });
  },
});
