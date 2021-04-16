import { contractSources } from "../tests/constants/contractSources";
import { getCompiled } from "../tests/util/contracts";

exports.mochaGlobalSetup = async function () {
  // First compile all contracts
  console.log("Making sure all contracts are compiled...");
  let keys = Object.keys(contractSources);
  for (let i = 0; i < keys.length; i++) {
    await getCompiled(keys[i]);
  }
  console.log("done");
};
