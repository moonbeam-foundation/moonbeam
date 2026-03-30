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

interface ReplayConfig {
  forkUrl: string;
  forkBlockHash: string;
  forkBlockNumber: number;
  replayFromBlock: number;
  replayBlockCount: number;
}

interface OriginalBlockData {
  blockNumber: number;
  ethReceipts: TransactionReceipt[];
  substrateEventsByExtrinsic: Map<number, { section: string; method: string; data: string }[]>;
  rawEthTransactions: string[];
  rawSubstrateExtrinsics: string[];
  ethTxHashOrder: string[];
}

interface ComparisonResult {
  blockNumber: number;
  ethTxMismatches: EthTxMismatch[];
  substrateMismatches: SubstrateEventMismatch[];
  passed: boolean;
}

interface EthTxMismatch {
  txHash: string;
  field: string;
  original: string;
  replayed: string;
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function loadReplayConfig(): Promise<ReplayConfig> {
  const raw = await fs.readFile("tmp/replayBlockConfig.json", "utf-8");
  return JSON.parse(raw);
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

  if (originalReceipts.length !== replayedReceipts.length) {
    mismatches.push({
      txHash: "N/A",
      field: "receipt_count",
      original: String(originalReceipts.length),
      replayed: String(replayedReceipts.length),
    });
    return mismatches;
  }

  for (let i = 0; i < originalReceipts.length; i++) {
    const orig = originalReceipts[i];
    const repl = replayedReceipts[i];

    if (orig.status !== repl.status) {
      mismatches.push({
        txHash: orig.transactionHash,
        field: "status",
        original: orig.status,
        replayed: repl.status,
      });
    }

    if (orig.gasUsed !== repl.gasUsed) {
      mismatches.push({
        txHash: orig.transactionHash,
        field: "gasUsed",
        original: String(orig.gasUsed),
        replayed: String(repl.gasUsed),
      });
    }

    if (orig.logs.length !== repl.logs.length) {
      mismatches.push({
        txHash: orig.transactionHash,
        field: "logs_count",
        original: String(orig.logs.length),
        replayed: String(repl.logs.length),
      });
      continue;
    }

    for (let j = 0; j < orig.logs.length; j++) {
      const oLog = orig.logs[j];
      const rLog = repl.logs[j];

      if (oLog.address.toLowerCase() !== rLog.address.toLowerCase()) {
        mismatches.push({
          txHash: orig.transactionHash,
          field: `log[${j}].address`,
          original: oLog.address,
          replayed: rLog.address,
        });
      }

      const oTopics = ((oLog as any).topics ?? []).join(",");
      const rTopics = ((rLog as any).topics ?? []).join(",");
      if (oTopics !== rTopics) {
        mismatches.push({
          txHash: orig.transactionHash,
          field: `log[${j}].topics`,
          original: oTopics,
          replayed: rTopics,
        });
      }

      if (oLog.data !== rLog.data) {
        mismatches.push({
          txHash: orig.transactionHash,
          field: `log[${j}].data`,
          original: oLog.data,
          replayed: rLog.data,
        });
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
      replayConfig = await loadReplayConfig();
      log(
        `Replay config: blocks ${replayConfig.replayFromBlock}..${replayConfig.replayFromBlock + replayConfig.replayBlockCount - 1}`
      );
      log(`Fork point   : block #${replayConfig.forkBlockNumber} (${replayConfig.forkBlockHash})`);
      log(`Live RPC     : ${replayConfig.forkUrl}`);

      // ── Live chain connections ──
      const provider = providerForUrl(replayConfig.forkUrl);
      const { ApiPromise: ApiPromiseClass } = await import("@polkadot/api");
      liveApi = await ApiPromiseClass.create({ provider, noInitWarn: true });

      // ── Local lazy-loading node (new runtime) ──
      localApi = context.polkadotJs();

      const localSpecVersion = localApi.consts.system.version.specVersion.toNumber();
      const liveSpecVersion = liveApi.consts.system.version.specVersion.toNumber();
      log(`Live spec_version  : ${liveSpecVersion}`);
      log(`Local spec_version : ${localSpecVersion}`);

      // The lazy-loading node creates an empty init block at (forkBlock + 1)
      // so our first createBlock() produces block (forkBlock + 2).
      const localHead = (await localApi.rpc.chain.getHeader()).number.toNumber();
      log(`Local head block   : #${localHead} (first sealed block will be #${localHead + 1})`);
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
              `${original.rawSubstrateExtrinsics.length} substrate extrinsics`
          );

          // 2. Submit Ethereum transactions
          let ethSubmitted = 0;
          for (const rawTx of original.rawEthTransactions) {
            try {
              await context.viem().request({
                method: "eth_sendRawTransaction" as any,
                params: [rawTx as any],
              });
              ethSubmitted++;
            } catch (e: any) {
              log(`  ⚠️  eth_sendRawTransaction failed: ${(e.message ?? "").substring(0, 120)}`);
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

          // 6. Compare
          const ethMismatches = compareEthReceipts(original.ethReceipts, replayedReceipts);
          const substrateMismatches = skipSubstrate
            ? []
            : compareSubstrateEvents(
                original.substrateEventsByExtrinsic,
                replayedEventsByExtrinsic
              );

          const passed = ethMismatches.length === 0 && substrateMismatches.length === 0;
          results.push({
            blockNumber: blockNum,
            ethTxMismatches: ethMismatches,
            substrateMismatches,
            passed,
          });

          if (passed) {
            log(`  ✅ Block #${blockNum} — PASSED`);
          } else {
            log(`  ❌ Block #${blockNum} — FAILED`);
            for (const m of ethMismatches) {
              log(
                `     ETH ${m.txHash.substring(0, 18)}… | ${m.field}: ${m.original} → ${m.replayed}`
              );
            }
            for (const m of substrateMismatches) {
              log(`     SUB ext[${m.extrinsicIndex}] | ${m.description}`);
            }
          }
        }

        // ── Persist full results ──
        await fs.mkdir("tmp", { recursive: true });
        await fs.writeFile("tmp/replayResults.json", JSON.stringify(results, null, 2));

        // ── Summary ──
        const passedCount = results.filter((r) => r.passed).length;
        const failedCount = results.filter((r) => !r.passed).length;

        log(`\n${"═".repeat(70)}`);
        log(`  RESULT: ${passedCount} passed, ${failedCount} failed / ${results.length} blocks`);
        log(`  Full report: tmp/replayResults.json`);
        log(`${"═".repeat(70)}\n`);

        if (failedCount > 0) {
          log("📋 Ethereum mismatches:");
          for (const r of results.filter((r) => r.ethTxMismatches.length > 0)) {
            for (const m of r.ethTxMismatches) {
              log(`  #${r.blockNumber} | ${m.txHash} | ${m.field}: ${m.original} → ${m.replayed}`);
            }
          }
          log("📋 Substrate mismatches:");
          for (const r of results.filter((r) => r.substrateMismatches.length > 0)) {
            for (const m of r.substrateMismatches) {
              log(`  #${r.blockNumber} | ext[${m.extrinsicIndex}] | ${m.description}`);
            }
          }
        }

        const totalEthMismatches = results.reduce((s, r) => s + r.ethTxMismatches.length, 0);
        const totalSubMismatches = results.reduce((s, r) => s + r.substrateMismatches.length, 0);

        expect(
          totalEthMismatches,
          `${totalEthMismatches} Ethereum tx mismatch(es) across ${failedCount} block(s)`
        ).toBe(0);

        if (!skipSubstrate) {
          expect(
            totalSubMismatches,
            `${totalSubMismatches} Substrate event mismatch(es) across ${failedCount} block(s)`
          ).toBe(0);
        }
      },
    });
  },
});
