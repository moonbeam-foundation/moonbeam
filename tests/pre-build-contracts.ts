import solc from "solc";
import fs from "fs/promises";
import { contractSources } from "./contracts/sources";
import { Compiled } from "./tests/util/contracts";

function compileSolidity(contractContent: string, contractName: string = "Test"): Compiled {
  let result = JSON.parse(
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
      })
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
  await Promise.all(Object.keys(contractSources).map(compile));

  // Forcing exit to avoid solc maintaining the process
  process.exit(0);
};

main();
