import { contractSources } from "../tests/constants/contractSources";
import { compile } from "../tests/util/contracts";

exports.mochaGlobalSetup = async function () {
  // First compile all contracts
  console.log("Making sure all contracts are compiled...");
  await Promise.all(Object.keys(contractSources).map(compile));
  console.log("Done");
};
