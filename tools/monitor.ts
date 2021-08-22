// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";

import { getMonitoredApiFor, NETWORK_NAMES } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    url: {
      type: "string",
      description: "Websocket url",
      conflicts: ["networks"],
      string: true,
    },
    networks: {
      type: "array",
      choices: NETWORK_NAMES,
      description: "Known networks",
      string: true,
    },
  })
  .check(function (argv) {
    if (!argv.url && !argv.networks) {
      throw new Error("Error: must provide --url or --network");
    }
    return true;
  }).argv;

const main = async () => {
  if (argv.networks) {
    argv.networks.map(getMonitoredApiFor);
  } else {
    getMonitoredApiFor(argv.url);
  }
};

main();
