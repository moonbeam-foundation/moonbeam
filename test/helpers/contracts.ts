import fs from "fs";
import path from "path";

export interface Compiled {
  byteCode: string;
  contract: any;
  sourceCode: string;
}

export function getAllContracts(): string[] {
  const contractsPath = path.join(__dirname, `../contracts/compiled/`);
  const contracts = fs.readdirSync(contractsPath, { withFileTypes: true });
  // Register all the contract code
  return contracts
    .filter((dirent) => dirent.isFile())
    .map((contract) => path.basename(contract.name, ".json"));
}

const contracts: { [name: string]: Compiled } = {};
export function getCompiled(name: string): Compiled {
  if (
    !fs.existsSync(
      path.join(__dirname, "..", "..", "helpers", "compiled-contracts", `${name}.json`)
    )
  ) {
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  }
  if (!contracts[name]) {
    try {
      contracts[name] = require(`../contracts/compiled/${name}.json`);
    } catch (e) {
      throw new Error(
        `Contract name ${name} is not compiled. Please run 'npm run pre-build-contracts`
      );
    }
  }

  return contracts[name];
}
