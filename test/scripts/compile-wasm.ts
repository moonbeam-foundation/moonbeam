import fs from "fs/promises";
import path from "path";
import child_process from "child_process";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .options({
    OutputDirectory: {
      type: "string",
      alias: "o",
      description: "Output directory for compiled contracts",
      default: "precompiled-wasm",
    },
    Binary: {
      type: "string",
      alias: "b",
      description: "Moonbeam binary path",
      default: "contracts/src",
    },
    Chain: {
      type: "string",
      alias: "c",
      description: "runtime chain to use",
      require: true,
    },
    Verbose: {
      type: "boolean",
      alias: "v",
      description: "Verbose mode for extra logging.",
      default: false,
    },
  })
  .command("compile", "Compile wasm", async (argv) => {
    await main(argv as any);
  })
  .parse();

async function main(args: any) {
  const outputDirectory = path.join(process.cwd(), args.argv.OutputDirectory);
  const binaryPath = args.argv.Binary;

  console.log(`🗃️  Binary: ${binaryPath}`);
  console.log(`🗃️  Output directory: ${outputDirectory}`);

  child_process.execSync(`mkdir -p ${outputDirectory}`);

  await fs.mkdir("tmp", { recursive: true });
  const tmpDir = await fs.mkdtemp("tmp/base-path");
  try {
    const command =
      `${binaryPath} precompile-wasm --log=wasmtime-runtime --base-path=${tmpDir} ` +
      `--chain ${args.argv.Chain} ${outputDirectory}`;
    console.log(`🗃️  ${command}`);

    child_process.execSync(`${command}`);
  } finally {
    if ((await fs.stat(tmpDir)).isDirectory()) {
      await fs.rm(tmpDir, { recursive: true, force: true });
    }
  }
}
