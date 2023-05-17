import fs from "fs";
import path from "path";

export interface Compiled {
  byteCode: `0x${string}`;
  contract: ContractObject;
  sourceCode: string;
}

export interface ContractObject {
  abi: any[]
  devdoc: any
  evm: any
  ewasm: any
  metadata: any
  storageLayout: any
  userdoc: any
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
  const filePath = path.join(process.cwd(), "helpers", "compiled", `${name}.json`);
  if (!fs.existsSync(filePath)) {
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  }
  if (!contracts[name]) {
    try {
      const json = fs.readFileSync(filePath, "utf8");
      contracts[name] = JSON.parse(json);
    } catch (e) {
      throw new Error(
        `Contract name ${name} is not compiled. Please run 'npm run pre-build-contracts`
      );
    }
  }

  return contracts[name];
}
