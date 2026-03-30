/**
 * Prepare state overrides for block replay testing.
 *
 * Usage:
 *   tsx prepare-block-replay.ts process <baseOverridesPath> <outputPath> <runtimePath>
 *
 * Env vars:
 *   REPLAY_FORK_URL  – RPC endpoint of the chain to fork (default: https://rpc.api.moonbeam.network)
 *   REPLAY_BLOCK     – Block number to start replaying FROM (fork happens at REPLAY_BLOCK - 1)
 *
 * The script:
 *  1. Fetches the block hash of (REPLAY_BLOCK - 1) from the live chain
 *  2. Writes a small JSON sidecar (tmp/replayBlockConfig.json) consumed by the test
 *  3. Extends the lazy-loading state-overrides with the authorized-upgrade key
 */

import fs from "node:fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { convertExponentials } from "@zombienet/utils";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import jsonBg from "json-bigint";

const JSONbig = jsonBg({ useNativeBigInt: true });

async function rpcCall(url: string, method: string, params: any[] = []): Promise<any> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const json = await res.json();
  if (json.error) throw new Error(`RPC error: ${JSON.stringify(json.error)}`);
  return json.result;
}

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("1.0.0")
  .command(
    "process <inputPath> <outputPath> <runtimePath>",
    "Prepare overrides for block-replay test",
    (yargs) =>
      yargs
        .positional("inputPath", { describe: "Base state overrides JSON", type: "string" })
        .positional("outputPath", { describe: "Output overrides JSON", type: "string" })
        .positional("runtimePath", { describe: "Runtime WASM path", type: "string" }),
    async (argv) => {
      if (!argv.inputPath || !argv.outputPath || !argv.runtimePath) {
        throw new Error("All positional args are required");
      }

      const forkUrl = process.env.REPLAY_FORK_URL ?? "https://rpc.api.moonbeam.network";
      const replayBlock = Number(process.env.REPLAY_BLOCK ?? "0");
      if (replayBlock <= 0) {
        throw new Error("REPLAY_BLOCK env var must be set to a positive block number");
      }

      // Fork from the block BEFORE the one we want to replay
      const forkBlockNumber = replayBlock - 1;
      process.stdout.write(`Fetching block hash for #${forkBlockNumber} from ${forkUrl} ...`);
      const forkBlockHash = await rpcCall(forkUrl, "chain_getBlockHash", [forkBlockNumber]);
      process.stdout.write(` ${forkBlockHash}\n`);

      // Write replay config sidecar for the test to consume
      const replayConfig = {
        forkUrl,
        forkBlockHash,
        forkBlockNumber,
        replayFromBlock: replayBlock,
        replayBlockCount: Number(process.env.REPLAY_BLOCK_COUNT ?? "1"),
      };
      await fs.mkdir("tmp", { recursive: true });
      await fs.writeFile("tmp/replayBlockConfig.json", JSON.stringify(replayConfig, null, 2));
      process.stdout.write(`Wrote replay config to tmp/replayBlockConfig.json\n`);

      // Patch the moonwall config to set the fork blockHash dynamically.
      // moonwall passes `defaultForkConfig.blockHash` as `--block=<hash>` to the node.
      const moonwallConfigPath = "moonwall.config.json";
      const moonwallConfig = JSON.parse((await fs.readFile(moonwallConfigPath)).toString());
      const replayEnv = moonwallConfig.environments.find(
        (e: any) => e.name === "block_replay_moonbeam"
      );
      if (replayEnv) {
        const launchSpec = replayEnv.foundation.launchSpec[0];
        launchSpec.defaultForkConfig.blockHash = forkBlockHash;
        // Also update the fork URL if overridden
        launchSpec.defaultForkConfig.url = forkUrl;
        await fs.writeFile(moonwallConfigPath, JSON.stringify(moonwallConfig, null, 2));
        process.stdout.write(`Patched moonwall.config.json with blockHash=${forkBlockHash}\n`);
      } else {
        process.stdout.write(
          `⚠️  block_replay_moonbeam environment not found in moonwall.config.json\n`
        );
      }

      // Prepare state overrides (authorize the runtime upgrade)
      process.stdout.write(`Reading runtime from: ${argv.runtimePath} ...`);
      const runtimeBlob = await fs.readFile(argv.runtimePath);
      process.stdout.write("Done ✅\n");

      const runtimeHash = blake2AsHex(runtimeBlob);
      process.stdout.write(`Runtime hash: ${runtimeHash}\n`);

      process.stdout.write(`Reading base overrides from: ${argv.inputPath} ...`);
      const overrides = JSONbig.parse((await fs.readFile(argv.inputPath)).toString());
      process.stdout.write("Done ✅\n");

      // Authorize the runtime upgrade
      const storageKey = u8aToHex(
        u8aConcat(xxhashAsU8a("System", 128), xxhashAsU8a("AuthorizedUpgrade", 128))
      );
      overrides.push({ key: storageKey, value: `${runtimeHash}01` });

      process.stdout.write(`Writing overrides to: ${argv.outputPath} ...`);
      await fs.writeFile(
        argv.outputPath,
        convertExponentials(JSONbig.stringify(overrides, null, 3))
      );
      process.stdout.write("Done ✅\n");
    }
  )
  .parse();
