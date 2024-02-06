import fs from "fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { convertExponentials } from "@zombienet/utils";
import { blake2AsHex } from "@polkadot/util-crypto";
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
          optional: true,
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

      localRaw.genesis.raw.top = {
        ...localRaw.genesis.raw.top,
        "0x45323df7cc47150b3930e2666b0aa3132fa9f1bf25567808771bff091dc89ecd": `${runtimeHash}00`,
      };

      process.stdout.write(`Writing to: ${argv.outputPath} ...`);
      await fs.writeFile(
        argv.outputPath,
        convertExponentials(JSONbig.stringify(localRaw, null, 3))
      );
      process.stdout.write("Done ✅\n");
    }
  )
  .parse();
