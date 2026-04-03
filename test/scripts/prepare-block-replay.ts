/**
 * Prepare state overrides for block replay testing.
 *
 * Usage:
 *   tsx prepare-block-replay.ts process <baseOverridesPath> <outputPath> <runtimePath>
 *
 * The script extends the lazy-loading state-overrides with the
 * authorized-upgrade key so the new runtime is active on the forked node.
 */

import fs from "node:fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { convertExponentials } from "@zombienet/utils";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import jsonBg from "json-bigint";

const JSONbig = jsonBg({ useNativeBigInt: true });

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
