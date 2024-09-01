import fs from "fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { convertExponentials } from "@zombienet/utils";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import jsonBg from "json-bigint";

const JSONbig = jsonBg({ useNativeBigInt: true });

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .command(
    "process <inputPath> <outputPath> <runtimePath>",
    "Preapproves a runtime blob into a raw spec for easy upgrade",
    (yargs) => {
      return yargs
        .positional("inputPath", {
          describe: "Input path for plainSpecFile to modify",
          type: "string",
        })
        .positional("outputPath", {
          describe: "Output path for modified file",
          type: "string",
        })
        .positional("runtimePath", {
          describe: "Input path for runtime blob to ",
          type: "string",
        });
    },
    async (argv) => {
      if (!argv.inputPath) {
        throw new Error("Input path is required");
      }

      if (!argv.outputPath) {
        throw new Error("Output path is required");
      }

      if (!argv.runtimePath) {
        throw new Error("Runtime path is required");
      }

      process.stdout.write(`Reading from: ${argv.runtimePath} ...`);
      const runtimeBlob = await fs.readFile(argv.runtimePath);
      process.stdout.write("Done ✅\n");

      const runtimeHash = blake2AsHex(runtimeBlob);
      process.stdout.write(`Runtime hash: ${runtimeHash}\n`);

      process.stdout.write(`Reading from: ${argv.inputPath} ...`);
      const localRaw = JSONbig.parse((await fs.readFile(argv.inputPath)).toString());
      process.stdout.write("Done ✅\n");

      const storageKey = u8aToHex(
        u8aConcat(xxhashAsU8a("System", 128), xxhashAsU8a("AuthorizedUpgrade", 128))
      );

      localRaw.push(
        {
          "key": storageKey,
          "value": `${runtimeHash}01`, // 01 sets RT version check = true
        }
      )

      process.stdout.write(`Writing to: ${argv.outputPath} ...`);
      await fs.writeFile(
        argv.outputPath,
        convertExponentials(JSONbig.stringify(localRaw, null, 3))
      );
      process.stdout.write("Done ✅\n");
    }
  )
  .parse();
