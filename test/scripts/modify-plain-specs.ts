import fs from "fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { ALITH_ADDRESS } from "@moonwall/util";
import { convertExponentials } from "@zombienet/utils";
import jsonBg from "json-bigint";

const JSONbig = jsonBg({ useNativeBigInt: true });

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .command(
    "process <inputPath> <outputPath>",
    "Overwrites a plainSpec with Alith modifications",
    (yargs) => {
      return yargs
        .positional("inputPath", {
          describe: "Input path for plainSpecFile to modify",
          type: "string",
        })
        .positional("outputPath", {
          describe: "Output path for modified file",
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

      process.stdout.write(`Reading from: ${argv.inputPath} ...`);
      const plainSpec = JSONbig.parse((await fs.readFile(argv.inputPath)).toString());
      process.stdout.write("Done ✅\n");

      plainSpec.bootNodes = [];
      plainSpec.genesis.runtime.authorMapping.mappings = [
        ["5HEL3iLyDyaqmfibHXAXVzyQq4fBqLCHGMEYxZXgRAuhEKXX", ALITH_ADDRESS],
      ];
      plainSpec.genesis.runtime.openTechCommitteeCollective.members = [ALITH_ADDRESS];

      process.stdout.write(`Writing to: ${argv.outputPath} ...`);
      await fs.writeFile(
        argv.outputPath,
        convertExponentials(JSONbig.stringify(plainSpec, null, 3))
      );
      process.stdout.write("Done ✅\n");
    }
  )
  .parse();
