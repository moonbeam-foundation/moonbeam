import solc from "solc";
import fs from "fs/promises";
import { contractSources } from "../contracts/sources";
import { Compiled } from "../util/contracts";

function getImports(dependency: string) {
  if (contractSources[dependency]) {
    return { contents: contractSources[dependency] };
  }
  return { error: "Source not found" };
}

function compileSolidity(contractContent: string, contractName: string): Compiled {
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
  const contract = result.contracts["main.sol"][contractName];
  return {
    byteCode: "0x" + contract.evm.bytecode.object,
    contract,
    sourceCode: contractContent,
  };
}

// Shouldn't be run concurrently with the same 'name'
async function compile(name: string): Promise<Compiled> {
  if (!contractSources[name])
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);

  const contractCompiled = compileSolidity(contractSources[name], name);
  let compiled = JSON.stringify(contractCompiled);
  await fs.mkdir(`contracts/compiled`, { recursive: true });
  await fs.writeFile(`./contracts/compiled/${name}.json`, compiled, {
    flag: "w",
  });
  console.log("New compiled contract file has been saved!");
  return contractCompiled;
}

const main = async () => {
  for (let name of Object.keys(contractSources)) {
    console.log(`Compiling ${name}`);
    try {
      await compile(name);
    } catch (e) {
      console.log(`Can't process contract ${name}: ${e.msg || e}`);
      process.exit(1);
    }
  }

  // Forcing exit to avoid solc maintaining the process
  process.exit(0);
};

main();
