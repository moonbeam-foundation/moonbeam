import type { CompiledContract } from "@moonwall/cli";
import chalk from "chalk";
import fs from "node:fs/promises";
import path from "node:path";
import solc from "solc";
import type { Abi } from "viem";
import crypto from "node:crypto";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const sourceByReference = {} as { [ref: string]: string };
const countByReference = {} as { [ref: string]: number };
const refByContract = {} as { [contract: string]: string };
let contractMd5 = {} as { [contract: string]: string };
const solcVersion = solc.version();

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .options({
    PreCompilesDirectory: {
      type: "string",
      alias: "p",
      description: "path to directory containing Precompile solidity files",
      default: "../precompiles/",
    },
    OutputDirectory: {
      type: "string",
      alias: "o",
      description: "Output directory for compiled contracts",
      default: "contracts/out",
    },
    SourceDirectory: {
      type: "string",
      alias: "i",
      description: "Source directory for solidity contracts to compile",
      default: "contracts/src",
    },
    Verbose: {
      type: "boolean",
      alias: "v",
      description: "Verbose mode for extra logging.",
      default: false,
    },
  })
  .command("compile", "Compile contracts", async (argv) => {
    await main(argv as any);
  })
  .parse();

async function main(args: any) {
  const precompilesPath = path.join(process.cwd(), args.argv.PreCompilesDirectory);
  const outputDirectory = path.join(process.cwd(), args.argv.OutputDirectory);
  const sourceDirectory = path.join(process.cwd(), args.argv.SourceDirectory);
  const tempFile = path.join(process.cwd(), args.argv.OutputDirectory, ".compile.tmp");

  console.log(`🧪  Solc version: ${solcVersion}`);
  console.log(`🗃️  Precompiles path: ${precompilesPath}`);
  console.log(`🗃️  Output directory: ${outputDirectory}`);
  console.log(`🗃️  Source directory: ${sourceDirectory}`);

  // Order is important so precompiles are available first
  const contractSourcePaths = [
    ...(await fs.readdir(precompilesPath)).map((filename) => ({
      filepath: path.join(precompilesPath, filename),
      // Solidity import removes the "../../.." when searching for imports
      importPath: /precompiles.*/.exec(path.join(precompilesPath, filename))![0],
      compile: true,
    })),
    {
      filepath: path.join(process.cwd(), "node_modules/@openzeppelin"),
      importPath: "@openzeppelin", // Reference in contracts are local
      compile: false,
    },
    {
      filepath: sourceDirectory,
      importPath: "", // Reference in contracts are local
      compile: true,
    },
  ];

  const sourceToCompile = {};
  const filePaths: string[] = [];
  for (const contractPath of contractSourcePaths) {
    const contracts = (await getFiles(contractPath.filepath)).filter((filename) =>
      filename.endsWith(".sol")
    );
    for (const filepath of contracts) {
      const ref = filepath
        .replace(contractPath.filepath, contractPath.importPath)
        .replace(/^\//, "");
      filePaths.push(filepath);
      sourceByReference[ref] = (await fs.readFile(filepath)).toString();
      if (contractPath.compile) {
        countByReference[ref] = 0;
        if (!sourceByReference[ref].includes("// skip-compilation")) {
          sourceToCompile[ref] = sourceByReference[ref];
        }
      }
    }
  }

  console.log(`📁 Found ${Object.keys(sourceToCompile).length} contracts to compile`);
  const contractsToCompile: string[] = [];
  const tempFileExists = await fs
    .access(tempFile)
    .then(() => true)
    .catch(() => false);

  if (tempFileExists) {
    contractMd5 = JSON.parse((await fs.readFile(tempFile)).toString());
    for (const contract of Object.keys(sourceToCompile)) {
      const path = filePaths.find((path) => path.includes(contract));
      const contractHash = computeHash((await fs.readFile(path!)).toString());
      if (contractHash !== contractMd5[contract]) {
        console.log(`  - Change in ${chalk.yellow(contract)}, compiling ⚙️`);
        contractsToCompile.push(contract);
      } else if (args.argv.Verbose) {
        console.log(`  - No change to ${chalk.green(contract)}, skipping ✅`);
      }
    }
  } else {
    console.log(`  - ${chalk.yellow("No temp file found, compiling all contracts ⚙️")}`);
    contractsToCompile.push(...Object.keys(sourceToCompile));
  }

  // Compile contracts
  for (const ref of contractsToCompile) {
    try {
      await compile(ref, outputDirectory);

      await fs.writeFile(tempFile, JSON.stringify(contractMd5, null, 2));
    } catch (e: any) {
      console.log(`Failed to compile: ${ref}`);
      if (e.errors) {
        for (const error of e.errors) {
          console.log(error.formattedMessage);
        }
      } else {
        console.log(e);
      }
      process.exit(1);
    }
  }
  // for (const ref of Object.keys(countByReference)) {
  //   if (!countByReference[ref]) {
  //     console.log(`${chalk.red("Warning")}: ${ref} never used: ${countByReference[ref]}`);
  //   }
  // }
}

// For some reasons, solc doesn't provide the relative path to imports :(
const getImports = (fileRef: string) => (dependency: string) => {
  if (sourceByReference[dependency]) {
    countByReference[dependency] = (countByReference[dependency] || 0) + 1;
    return { contents: sourceByReference[dependency] };
  }
  let base = fileRef;
  while (base && base.length > 1) {
    const localRef = path.join(base, dependency);
    if (sourceByReference[localRef]) {
      countByReference[localRef] = (countByReference[localRef] || 0) + 1;
      return { contents: sourceByReference[localRef] };
    }
    base = path.dirname(base);
  }
  return { error: "Source not found" };
};

function compileSolidity(
  fileRef: string,
  contractContent: string
): { [name: string]: CompiledContract<Abi> } {
  const filename = path.basename(fileRef);
  const result = JSON.parse(
    solc.compile(
      JSON.stringify({
        language: "Solidity",
        sources: {
          [filename]: {
            content: contractContent,
          },
        },
        settings: {
          optimizer: { enabled: true, runs: 200 },
          outputSelection: {
            "*": {
              "*": ["*"],
            },
          },
          debug: {
            revertStrings: "debug",
          },
        },
      }),
      { import: getImports(fileRef) }
    )
  );
  if (!result.contracts) {
    throw result;
  }
  return Object.keys(result.contracts[filename]).reduce(
    (p, contractName) => {
      p[contractName] = {
        byteCode:
          `0x${result.contracts[filename][contractName].evm.bytecode.object}` as `0x${string}`,
        contract: result.contracts[filename][contractName],
        sourceCode: contractContent,
      };
      return p;
    },
    {} as { [name: string]: CompiledContract<Abi> }
  );
}

// Shouldn't be run concurrently with the same 'name'
async function compile(
  fileRef: string,
  destPath: string
): Promise<{ [name: string]: CompiledContract<Abi> }> {
  const soliditySource = sourceByReference[fileRef];
  countByReference[fileRef]++;
  if (!soliditySource) {
    throw new Error(`Missing solidity file: ${fileRef}`);
  }
  const compiledContracts = compileSolidity(fileRef, soliditySource);

  await Promise.all(
    Object.keys(compiledContracts).map(async (contractName) => {
      const dest = `${path.join(destPath, path.dirname(fileRef), contractName)}.json`;
      if (refByContract[dest]) {
        console.warn(
          chalk.red(
            `Contract ${contractName} already exist from ${refByContract[dest]}. Erasing previous version`
          )
        );
      }
      await fs.mkdir(path.dirname(dest), { recursive: true });
      await fs.writeFile(dest, JSON.stringify(compiledContracts[contractName], null, 2), {
        flag: "w",
        encoding: "utf-8",
      });
      console.log(`  - ${chalk.green(`${contractName}.json`)} file has been saved 💾`);
      refByContract[dest] = fileRef;
      contractMd5[fileRef] = computeHash(soliditySource);
    })
  );
  return compiledContracts;
}

async function getFiles(dir) {
  const subdirs = await fs.readdir(dir);
  const files = await Promise.all(
    subdirs.map(async (subdir) => {
      const res = path.resolve(dir, subdir);
      return (await fs.stat(res)).isDirectory() ? getFiles(res) : res;
    })
  );
  return files.reduce((a, f) => a.concat(f), []);
}

function computeHash(input: string): string {
  const hash = crypto.createHash("md5");
  hash.update(input + solcVersion);
  return hash.digest("hex");
}
