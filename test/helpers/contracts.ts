import fs from "fs";
import path from "path";
import type { Abi, AbiParameter, Address, Narrow } from "abitype";
import { type } from "os";

// export interface Compiled {
//   byteCode: `0x${string}`;
//   contract: ContractObject;
//   sourceCode: string;
// }

export type CompiledContract<TAbi extends Abi> = {
  byteCode: `0x${string}`;
  contract: ContractObject<TAbi>;
  sourceCode: string;
};

// export interface ContractObject {
//   abi: AbiItem[];
//   devdoc: any;
//   evm: any;
//   ewasm: any;
//   metadata: any;
//   storageLayout: any;
//   userdoc: any;
// }
export type ContractObject<TAbi extends Abi> = {
  abi: TAbi;
  devdoc: any;
  evm: any;
  ewasm: any;
  metadata: any;
  storageLayout: any;
  userdoc: any;
};

export function getAllContracts(): string[] {
  const contractsPath = path.join(__dirname, `../contracts/compiled/`);
  const contracts = fs.readdirSync(contractsPath, { withFileTypes: true });
  // Register all the contract code
  return contracts
    .filter((dirent) => dirent.isFile())
    .map((contract) => path.basename(contract.name, ".json"));
}
type Contracts<T extends Abi> = { [name: string]: CompiledContract<T> };
const contracts: Contracts<Abi> = {};

export function getCompiled<TAbi extends Abi>(name: string): CompiledContract<TAbi> {
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
  // @ts-expect-error
  return contracts[name];
}
