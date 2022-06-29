import solc from "solc";
import chalk from "chalk";
import fs from "fs/promises";
import path from "path";
import { Compiled } from "../util/contracts";

let sourceByReference = {} as { [ref: string]: string };
let refByContract = {} as { [contract: string]: string };

function getImports(dependency: string) {
  if (sourceByReference[dependency]) {
    return { contents: sourceByReference[dependency] };
  }
  return { error: "Source not found" };
}

function compileSolidity(contractContent: string): { [name: string]: Compiled } {
  const result = JSON.parse(
    solc.compile(
      JSON.stringify({
        language: "Solidity",
        sources: {
          "main.sol": {
            content: contractContent,
          },
        },
        settings: {
          outputSelection: {
            "*": {
              "*": ["*"],
            },
          },
        },
      }),
      { import: getImports }
    )
  );
  if (!result.contracts) {
    throw result;
  }
  return Object.keys(result.contracts["main.sol"]).reduce((p, contractName) => {
    p[contractName] = {
      byteCode: "0x" + result.contracts["main.sol"][contractName].evm.bytecode.object,
      contract: result.contracts["main.sol"][contractName],
      sourceCode: contractContent,
    };
    return p;
  }, {} as { [name: string]: Compiled });
}

// Shouldn't be run concurrently with the same 'name'
async function compile(fileRef: string): Promise<{ [name: string]: Compiled }> {
  const soliditySource = sourceByReference[fileRef];
  if (!soliditySource) {
    throw new Error(`Missing solidity file: ${fileRef}`);
  }
  const compiledContracts = compileSolidity(soliditySource);

  console.debug(`Processing file: ${fileRef}`);
  await Promise.all(
    Object.keys(compiledContracts).map(async (contractName) => {
      if (refByContract[contractName]) {
        console.warn(
          chalk.red(
            `Contract ${contractName} already exist from ` +
              `${refByContract[contractName]}. ` +
              `Erasing previous version`
          )
        );
      }
      await fs.mkdir(`contracts/compiled`, { recursive: true });
      await fs.writeFile(
        `./contracts/compiled/${contractName}.json`,
        JSON.stringify(compiledContracts[contractName]),
        {
          flag: "w",
        }
      );
      console.log(`  - ${chalk.green(`${contractName}.json`)} file has been saved!`);
      refByContract[contractName] = fileRef;
    })
  );
  return compiledContracts;
}

const main = async () => {
  const precompilesPath = path.join(__dirname, "../../precompiles");
  // Order is important so precompiles are available first
  const contractSourcePaths = [
    ...(await fs.readdir(precompilesPath)).map((filename) => ({
      filepath: path.join(precompilesPath, filename),
      // Solidity import removes the "../../.." when searching for imports
      importPath: /precompiles.*/.exec(path.join(precompilesPath, filename))[0],
    })),
    {
      filepath: path.join(__dirname, "../contracts/solidity"),
      importPath: "", // Reference in contracts are local
    },
  ];

  for (const contractPath of contractSourcePaths) {
    const contracts = (await fs.readdir(contractPath.filepath)).filter((filename) =>
      filename.endsWith(".sol")
    );
    for (let filename of contracts) {
      sourceByReference[path.join(contractPath.importPath, filename)] = (
        await fs.readFile(path.join(contractPath.filepath, filename))
      ).toString();
    }
  }

  // Compile contracts
  for (const ref of Object.keys(sourceByReference)) {
    try {
      await compile(ref);
    } catch (e) {
      console.log(`Failed to compile: ${ref}`);
      if (e.errors) {
        e.errors.forEach((error) => {
          console.log(error.formattedMessage);
        });
      } else {
        console.log(e);
      }
      process.exit(1);
    }
  }

  // Forcing exit to avoid solc maintaining the process
  process.exit(0);
};

main();
