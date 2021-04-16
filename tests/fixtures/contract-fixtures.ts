import { contractSources } from "../tests/constants/contractSources";
import { getCompiled } from "../tests/util/contracts";

exports.mochaGlobalSetup = async function () {
  // First compile all contracts
  console.log("Making sure all contracts are compiled...");
  await Promise.all(Object.keys(contractSources).map(getCompiled));
  console.log("done");
};
