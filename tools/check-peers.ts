// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";

import { getApiFor, isKnownNetwork, NETWORK_COLORS, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
  }).argv;

const main = async () => {
  const api = await getApiFor(argv);

  const peers = await api.rpc.system.peers();
  console.log(peers);
};

main();
