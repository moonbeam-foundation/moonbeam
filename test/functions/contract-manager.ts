import fs from "fs";
import path from "path";

/**
 * Compiled contract
 * @interface Compiled
 * @property {string} byteCode - The contract byte code
 * @property {any} contract - The contract object with the ABI
 * @property {string} sourceCode - The contract source code
 */
export interface Compiled {
  byteCode: string;
  contract: any;
  sourceCode: string;
}

// Path to the compiled contracts/
export const COMPILED_CONTRACT_PATH = `../contracts/compiled/`;

// Map of all the compiled contracts by name
export const contracts: { [name: string]: Compiled } = {};

/**
 * @description - Get all the compiled contracts
 * @returns {string[]} - List of all the compiled contracts
 */
export function getAllContracts(): string[] {
  const contractsPath = path.join(__dirname, COMPILED_CONTRACT_PATH);
  const contracts = fs.readdirSync(contractsPath);
  return contracts.map((contract) => path.basename(contract, ".json"));
}

/**
 * @description - Get the compiled contract by name
 * @param {string} name - The name of the contract
 * @returns {Compiled} - The compiled contract
 */
export function getCompiled(name: string): Compiled {
  if (!fs.existsSync(path.join(__dirname, COMPILED_CONTRACT_PATH, `${name}.json`))) {
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  }
  if (!contracts[name]) {
    try {
      contracts[name] = require(path.join(COMPILED_CONTRACT_PATH, `${name}.json`));
    } catch (e) {
      // TODO: The command line is not valid anymore, needs to be updated
      throw new Error(
        `Contract name ${name} is not compiled. Please run 'npm run pre-build-contracts`
      );
    }
  }

  return contracts[name];
}
