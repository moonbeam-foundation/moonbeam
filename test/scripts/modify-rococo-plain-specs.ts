import fs from "fs/promises";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { convertExponentials } from "@zombienet/utils";
import jsonBg from "json-bigint";

const JSONbig = jsonBg({ useNativeBigInt: true });

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .command(
    `process <inputPath> <outputPath>`,
    "Overwrites a rococo local plainSpec to enable async backing",
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
      process.stdout.write(`Reading from: ${argv.inputPath} ...`);
      const plainSpec = JSONbig.parse((await fs.readFile(argv.inputPath!)).toString());
      process.stdout.write(`Done ✅\n`);

      plainSpec.genesis.runtime.configuration.config.asyncBackingParams.maxCandidateDepth = 3;
      plainSpec.genesis.runtime.configuration.config.asyncBackingParams.allowedAncestryLen = 2;
      plainSpec.genesis.runtime.configuration.config.schedulingLookahead = 2;

      process.stdout.write(`Writing to: ${argv.outputPath} ...`);
      await fs.writeFile(
        argv.outputPath!,
        convertExponentials(JSONbig.stringify(plainSpec, null, 3))
      );
      process.stdout.write(`Done ✅\n`);
    }
  )
  .parse();
